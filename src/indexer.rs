//! Utilities for handling files and directories.

use anyhow::Result;
use glob::Pattern;
use indicatif::ProgressIterator;
use markdown::mdast;
use petgraph::graph::UnGraph;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::meilisearch::Meilisearch;
use crate::parsing::block::{Block, BlockBuilder};
use crate::parsing::file::{File, FileBuilder};

/// Walks a directory tree and yields files matching a glob pattern.
pub struct MdWalker {
    /// The underlying directory walker.
    walker: walkdir::IntoIter,
    /// The glob pattern to match.
    pattern: Pattern,
}

impl MdWalker {
    /// Create a new `GlobWalker` with a given path and pattern.
    pub fn new(path: &str) -> MdWalker {
        MdWalker {
            walker: WalkDir::new(path).into_iter(),
            pattern: Pattern::new("*.md").unwrap(),
        }
    }
}

impl Iterator for MdWalker {
    type Item = Result<(PathBuf, mdast::Node, String)>;

    /// Get the next file matching the pattern. Returns the markdown AST.
    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.walker.by_ref() {
            match entry {
                Ok(e) if self.pattern.matches_path(e.path()) => {
                    let content = match std::fs::read_to_string(e.path()) {
                        Ok(content) => content,
                        Err(msg) => return Some(Err(msg.into())),
                    };
                    let ast = markdown::to_mdast(&content, &markdown::ParseOptions::default());
                    match ast {
                        Ok(ast) => return Some(Ok((e.path().to_path_buf(), ast, content))),
                        Err(msg) => return Some(Err(anyhow::Error::msg(msg.to_string()))),
                    };
                }
                Err(msg) => return Some(Err(msg.into())),
                Ok(_) => continue,
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum GraphNode {
    File { id: String, title: Option<String> },
    Block { id: String },
}

pub struct Indexer {
    pub db: Meilisearch,
    pub graph: UnGraph<GraphNode, ()>,
}

impl Indexer {
    pub async fn new() -> Indexer {
        Indexer {
            db: Meilisearch::new().await,
            graph: UnGraph::default(),
        }
    }

    pub async fn index_files(&mut self, path: &str, index_blocks: bool) -> Result<()> {
        // An index is where the documents are stored.
        let files = self.db.client.index("files");
        let walker = MdWalker::new(path);
        let mut tasks = vec![];
        for file in walker
            .into_iter()
            .collect::<Vec<Result<(PathBuf, mdast::Node, String)>>>()
            .into_iter()
            .progress()
        {
            let doc = match file {
                Ok((path, ast, content)) => {
                    let file = FileBuilder::new()
                        .with_path(path.clone())
                        .build(&content, &ast)?;
                    file.add_to_graph(&mut self.graph);
                    if index_blocks {
                        self.index_blocks(&ast, &content, file.id.clone(), path)
                            .await?;
                    }
                    file
                }
                Err(msg) => return Err(msg),
            };
            let task = files.add_documents(&[doc], Some("id")).await?;
            tasks.push(task);
        }
        for task in tasks {
            task.wait_for_completion(&self.db.client, None, None)
                .await?;
        }
        self.graph_link().await?;
        Ok(())
    }

    async fn index_blocks(
        &mut self,
        ast: &mdast::Node,
        content: &str,
        file_id: String,
        file_path: PathBuf,
    ) -> Result<()> {
        let blocks_index = self.db.client.index("blocks");
        let mut tasks = vec![];

        for child in ast.children().unwrap_or(&vec![]).iter() {
            if let mdast::Node::List(list) = child {
                for item in list.children.iter() {
                    if let mdast::Node::ListItem(list_item) = item {
                        let new_blocks = BlockBuilder::new()
                            .with_file_id(file_id.clone())
                            .with_file_path(file_path.clone())
                            .build(content, list_item)?;
                        for block in new_blocks.iter() {
                            block.add_to_graph(&mut self.graph)
                        }
                        let task_info = blocks_index.add_documents(&new_blocks, Some("id")).await?;
                        tasks.push(task_info);
                    }
                }
            } else if let mdast::Node::ListItem(list_item) = child {
                let new_blocks = BlockBuilder::new()
                    .with_file_id(file_id.clone())
                    .with_file_path(file_path.clone())
                    .build(content, list_item)?;
                let task_info = blocks_index.add_documents(&new_blocks, Some("id")).await?;
                tasks.push(task_info);
            }
        }
        for task in tasks {
            task.wait_for_completion(&self.db.client, None, None)
                .await?;
        }
        Ok(())
    }

    async fn graph_link(&mut self) -> Result<()> {
        // Collect all relevant node identifiers first
        let mut block_ids = Vec::new();
        let mut file_ids = Vec::new();

        for node in self.graph.node_indices() {
            match self.graph[node].clone() {
                GraphNode::Block { id, .. } => block_ids.push(id),
                GraphNode::File { id, .. } => file_ids.push(id),
            }
        }

        // Process blocks
        let blocks_index = self.db.client.index("blocks");
        for id in block_ids {
            let block: Block = blocks_index.get_document(&id).await?;
            block.add_edges(&mut self.graph)?;
        }

        // Process files
        let files_index = self.db.client.index("files");
        for id in file_ids {
            let file: File = files_index.get_document(&id).await?;
            file.add_edges(&mut self.graph)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::{block::Block, file::File};

    use super::*;

    use std::collections::HashMap;

    #[tokio::test]
    async fn test_index_blocks() {
        let path = PathBuf::from("graph/pages/tests___parsing___blocks___hierarchy.md");
        let content = std::fs::read_to_string(&path).unwrap();
        let file_id = "test".to_string();
        let ast = markdown::to_mdast(&content, &markdown::ParseOptions::default()).unwrap();
        let db = Meilisearch::new().await;
        let blocks_index = db.client.index("blocks");
        blocks_index.delete_all_documents().await.unwrap();
        Indexer::new()
            .await
            .index_blocks(&ast, &content, file_id.clone(), path)
            .await
            .unwrap();
        let mut blocks = blocks_index.get_documents::<Block>().await.unwrap().results;
        assert_eq!(blocks.len(), 5);
        blocks.sort_by_key(|b| b.content.clone());
        println!(
            "{:?}",
            blocks
                .iter()
                .map(|b| b.content.clone())
                .collect::<Vec<String>>()
        );

        let content = "- Lorem".to_string();
        let block1 = blocks
            .get(
                blocks
                    .binary_search_by_key(&content, |b| b.content.clone())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            block1,
            &Block {
                id: block1.id.clone(),
                file_id: file_id.clone(),
                parent_block_id: None,
                content,
                properties: HashMap::new(),
                wikilinks: vec![],
                tags: vec![],
            }
        );
        let content = "- Ipsum".to_string();
        let block2 = blocks
            .get(
                blocks
                    .binary_search_by_key(&content, |b| b.content.clone())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            block2,
            &Block {
                id: block2.id.clone(),
                file_id: file_id.clone(),
                parent_block_id: Some(block1.id.clone()),
                content,
                properties: HashMap::new(),
                wikilinks: vec![],
                tags: vec![],
            }
        );
        let content = "- Dolor".to_string();
        let block3 = blocks
            .get(
                blocks
                    .binary_search_by_key(&content, |b| b.content.clone())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            block3,
            &Block {
                id: block3.id.clone(),
                file_id: file_id.clone(),
                parent_block_id: Some(block1.id.clone()),
                content,
                properties: HashMap::new(),
                wikilinks: vec![],
                tags: vec![],
            }
        );
        let content = "- Sit".to_string();
        let block4 = blocks
            .get(
                blocks
                    .binary_search_by_key(&content, |b| b.content.clone())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            block4,
            &Block {
                id: block4.id.clone(),
                file_id: file_id.clone(),
                parent_block_id: Some(block3.id.clone()),
                content: "- Sit".to_string(),
                properties: HashMap::new(),
                wikilinks: vec![],
                tags: vec![],
            }
        );
        let content = "- Amet".to_string();
        let block5 = blocks
            .get(
                blocks
                    .binary_search_by_key(&content, |b| b.content.clone())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            block5,
            &Block {
                id: block5.id.clone(),
                file_id: file_id.clone(),
                parent_block_id: None,
                content: "- Amet".to_string(),
                properties: HashMap::new(),
                wikilinks: vec![],
                tags: vec![],
            }
        );
    }

    #[tokio::test]
    async fn test_index_files() {
        let path = "graph/pages/";
        let db = Meilisearch::new().await;
        let files_index = db.client.index("files");
        files_index.delete_all_documents().await.unwrap();
        Indexer::new().await.index_files(path, false).await.unwrap();
        let files = files_index.get_documents::<File>().await.unwrap().results;
        assert!(!files.is_empty());

        let file = files
            .into_iter()
            .find(|f| f.path == "graph/pages/tests___parsing___files___basic.md")
            .unwrap();
        assert_eq!(
            file,
            File {
                id: file.id.clone(),
                path: "graph/pages/tests___parsing___files___basic.md".to_string(),
                title: "tests/parsing/files/basic".to_string(),
                properties: HashMap::from([("foo".to_string(), "bar".to_string())]),
                wikilinks: vec!["wikilink".to_string()],
                tags: vec![
                    "foo".to_string(),
                    "bar".to_string(),
                    "tag".to_string(),
                    "multi word tag".to_string()
                ],
            }
        );
    }
}
