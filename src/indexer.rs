//! Utilities for handling files and directories.

use glob::Pattern;
use indicatif::ProgressIterator;
use markdown::mdast;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::meilisearch::Meilisearch;
use crate::parsing::block::BlockBuilder;
use crate::parsing::file::FileBuilder;

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
    type Item = Result<(PathBuf, mdast::Node, String), String>;

    /// Get the next file matching the pattern. Returns the markdown AST.
    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.walker.by_ref() {
            match entry {
                Ok(e) if self.pattern.matches_path(e.path()) => {
                    let content = match std::fs::read_to_string(e.path()) {
                        Ok(content) => content,
                        Err(msg) => return Some(Err(msg.to_string())),
                    };
                    let ast = markdown::to_mdast(&content, &markdown::ParseOptions::default());
                    match ast {
                        Ok(ast) => return Some(Ok((e.path().to_path_buf(), ast, content))),
                        Err(msg) => return Some(Err(msg.to_string())),
                    }
                }
                Err(msg) => return Some(Err(msg.to_string())),
                Ok(_) => continue,
            }
        }
        None
    }
}

pub struct Indexer {
    pub db: Meilisearch,
}

impl Indexer {
    pub async fn new() -> Indexer {
        Indexer {
            db: Meilisearch::new().await,
        }
    }

    pub async fn index_files(&self, path: &str) -> Result<(), String> {
        // An index is where the documents are stored.
        let files = self.db.client.index("files");
        let walker = MdWalker::new(path);
        for file in walker
            .into_iter()
            .collect::<Vec<Result<(PathBuf, mdast::Node, String), String>>>()
            .into_iter()
            .progress()
        {
            let doc = match file {
                Ok((path, ast, content)) => {
                    let fb = FileBuilder::new();
                    self.index_blocks(&ast, &content, fb.get_id(), path.clone())
                        .await
                        .map_err(|e| e.to_string())?;

                    fb.with_path(path).with_ast(ast).build(&content)?
                }
                Err(msg) => return Err(msg.to_string()),
            };
            files
                .add_documents(&[doc], Some("id"))
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn index_blocks(
        &self,
        ast: &mdast::Node,
        content: &str,
        file_id: String,
        file_path: PathBuf,
    ) -> Result<(), String> {
        let blocks = self.db.client.index("blocks");
        for child in ast.children().unwrap_or(&vec![]).iter() {
            if let mdast::Node::ListItem(list_item) = child {
                let new_blocks = BlockBuilder::new()
                    .with_list_item(list_item.clone())
                    .with_file_id(file_id.clone())
                    .with_file_path(file_path.clone())
                    .build(content)?;
                blocks
                    .add_documents(&new_blocks, Some("id"))
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
