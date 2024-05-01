use std::{collections::HashMap, path::PathBuf};

use markdown::mdast::{ListItem, Node};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct BlockBuilder {
    list_item: Option<ListItem>,
    file_id: Option<String>,
    file_path: Option<PathBuf>,
    parent_block_id: Option<String>,
}

impl BlockBuilder {
    pub fn new() -> BlockBuilder {
        BlockBuilder {
            list_item: None,
            file_id: None,
            file_path: None,
            parent_block_id: None,
        }
    }

    pub fn with_list_item(mut self, list_item: ListItem) -> BlockBuilder {
        self.list_item = Some(list_item);
        self
    }

    pub fn with_file_id(mut self, file_id: String) -> BlockBuilder {
        self.file_id = Some(file_id);
        self
    }

    pub fn with_file_path(mut self, file_path: PathBuf) -> BlockBuilder {
        self.file_path = Some(file_path);
        self
    }

    pub fn with_parent_block_id(mut self, parent_block_id: String) -> BlockBuilder {
        self.parent_block_id = Some(parent_block_id);
        self
    }

    fn get_content(&self) -> Result<String, String> {
        let list_item = self.list_item.as_ref().expect("No list item");
        let file_path = self.file_path.clone().expect("No file path");
        let buf = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        let position = list_item.position.clone().unwrap();
        Ok(buf[position.start.offset..position.end.offset].to_string())
    }

    fn get_id(_content: &str) -> String {
        let re = Regex::new("id:: ([a-f0-9-]+)");
        let captures = re.unwrap().captures(_content).unwrap();
        match captures.get(1) {
            Some(id) => id.as_str().to_string(),
            None => uuid::Uuid::new_v4().to_string(),
        }
    }

    fn get_properties(_content: &str) -> HashMap<String, String> {
        let re = Regex::new(r"([a-z]+):: ([a-z]+)").unwrap();
        let mut properties = HashMap::new();
        for captures in re.captures_iter(_content) {
            assert_eq!(
                captures.len(),
                3,
                "There should be the full capture, a key, and a value: {:?}",
                captures
            );
            let k = captures[1].to_string();
            let v = captures[2].to_string();
            if k != "id" {
                properties.insert(k, v);
            }
        }
        properties
    }

    fn get_wikilinks(content: &str) -> Vec<String> {
        // [[something]] but not #[[something]]
        let re = Regex::new(r"\s\[\[([\w\s]+)\]\]").unwrap();
        let mut wikilinks = vec![];
        for captures in re.captures_iter(content) {
            assert_eq!(
                captures.len(),
                2,
                "There should be the full capture and the wikilink: {:?}",
                captures
            );
            wikilinks.push(captures[1].trim().to_string());
        }
        wikilinks
    }

    fn get_tags(content: &str) -> Vec<String> {
        // #something or #[[something]]
        let re = Regex::new(r"(?i)#\[\[([\w\s]+)\]\]|#(\w+)").unwrap();
        let mut tags = vec![];
        for captures in re.captures_iter(content) {
            assert_eq!(
                captures.len(),
                3,
                "There should be the full capture and the tag: {:?}",
                captures
            );
            if let Some(tag) = captures.get(1) {
                tags.push(tag.as_str().to_string());
            } else if let Some(tag) = captures.get(2) {
                tags.push(tag.as_str().to_string());
            } else {
                panic!("No tag found");
            }
        }
        tags
    }

    pub fn build(self) -> Result<Vec<Block>, String> {
        let content = self.get_content()?;
        let list_item = self.list_item.expect("No list item");
        let id = Self::get_id(&content);
        let properties = Self::get_properties(&content);
        let wikilinks = Self::get_wikilinks(&content);
        let tags = Self::get_tags(&content);
        let file_id = self.file_id.expect("No file id");
        let mut blocks = vec![];
        for child in list_item.children.iter() {
            if let Node::List(list) = child {
                for child in list.children.iter() {
                    if let Node::ListItem(list_item) = child {
                        let block = BlockBuilder::new()
                            .with_list_item(list_item.clone())
                            .with_file_id(file_id.clone())
                            .with_parent_block_id(id.clone())
                            .build()?;
                        blocks.extend(block);
                    }
                }
            }
        }
        let root = Block {
            id,
            content,
            file_id,
            properties,
            wikilinks,
            tags,
            parent_block_id: self.parent_block_id,
        };
        blocks.push(root);
        Ok(blocks)
    }
}

/// This is a logseq block, which is a markdown list element
#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    /// The index of the block in the list
    pub id: String,
    /// The text content of the block divided into types
    pub content: String,
    /// The file this block belongs to
    pub file_id: String,
    /// Parent block id
    pub parent_block_id: Option<String>,
    /// The block properties
    pub properties: HashMap<String, String>,
    /// The block tags
    pub tags: Vec<String>,
    /// The wikilinks in the block
    pub wikilinks: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod builder {
        use super::*;

        mod properties {
            use super::*;

            fn get_list_blocks_as_str() -> Vec<String> {
                // First lets get the list items from the markdown file
                let content =
                    std::fs::read_to_string("graph/pages/tests___parsing___blocks___property.md")
                        .unwrap();
                let ast = markdown::to_mdast(&content, &markdown::ParseOptions::default());
                let list_items: Vec<String> = ast
                    .unwrap()
                    .children()
                    .unwrap()
                    .iter()
                    .filter_map(|child| match child {
                        Node::List(list) => Some(list),
                        _ => None,
                    })
                    .flat_map(|list| list.children.iter())
                    .filter_map(|child| match child {
                        Node::ListItem(list_item) => Some(list_item),
                        _ => None,
                    })
                    .flat_map(|list_item| list_item.children.iter())
                    .filter_map(|child| match child {
                        Node::Paragraph(paragraph) => Some(paragraph),
                        _ => None,
                    })
                    .flat_map(|paragraph| paragraph.children.iter())
                    .filter_map(|child| match child {
                        Node::Text(text) => Some(text.value.clone()),
                        _ => None,
                    })
                    .collect();
                assert_eq!(
                    list_items.len(),
                    4,
                    "There should be 4 list items in this file"
                );
                list_items
            }

            #[test]
            fn test_get_content() {
                let content =
                    std::fs::read_to_string("graph/pages/tests___parsing___blocks___property.md")
                        .unwrap();
                let ast = markdown::to_mdast(&content, &markdown::ParseOptions::default()).unwrap();
                let list_items: Vec<&ListItem> = ast
                    .children()
                    .unwrap()
                    .iter()
                    .filter_map(|child| match child {
                        Node::List(list) => Some(list),
                        _ => None,
                    })
                    .flat_map(|list| list.children.iter())
                    .filter_map(|child| match child {
                        Node::ListItem(list_item) => Some(list_item),
                        _ => None,
                    })
                    .collect();
                let first = BlockBuilder::new()
                    .with_list_item(list_items[0].clone())
                    .with_file_path(
                        std::path::PathBuf::from(
                            "graph/pages/tests___parsing___blocks___property.md",
                        )
                        .into(),
                    )
                    .get_content();
                assert_eq!(
                    first.unwrap(),
                    "- This tests a block property\n  foo:: bar\n  id:: 662ef9e2-4b89-4f7d-9a54-afd395b03cb0"
                );
            }

            #[test]
            fn test_get_id() {
                let list_items = get_list_blocks_as_str();

                // The first and third items have an id
                let first = BlockBuilder::get_id(&list_items[0]);
                assert_eq!(first, "662ef9e2-4b89-4f7d-9a54-afd395b03cb0");
                let third = BlockBuilder::get_id(&list_items[2]);
                assert_eq!(third, "662effa7-a861-42df-a5bf-64c783eb8b64");
            }

            #[test]
            fn test_get_properties() {
                let list_items = get_list_blocks_as_str();

                // The first and second items have properties foo:: bar
                let first = BlockBuilder::get_properties(&list_items[0]);
                assert_eq!(first.get("foo"), Some(&"bar".to_string()));
                let second = BlockBuilder::get_properties(&list_items[1]);
                assert_eq!(second.get("foo"), Some(&"bar".to_string()));
                let third = BlockBuilder::get_properties(&list_items[2]);
                assert_eq!(third.len(), 0);
                let fourth = BlockBuilder::get_properties(&list_items[3]);
                assert_eq!(fourth.len(), 0);
            }

            #[test]
            fn test_get_properties_does_not_return_ids() {
                let list_items = get_list_blocks_as_str();

                for li in list_items {
                    let properties = BlockBuilder::get_properties(&li);
                    assert_eq!(properties.get("id"), None);
                }
            }
        }

        mod links {
            use super::*;

            fn get_content() -> String {
                let content = std::fs::read_to_string(
                    "graph/pages/tests___parsing___blocks___tags_wikilinks.md",
                )
                .unwrap();
                let ast = markdown::to_mdast(&content, &markdown::ParseOptions::default()).unwrap();
                let list_items: Vec<&ListItem> = ast
                    .children()
                    .unwrap()
                    .iter()
                    .filter_map(|child| match child {
                        Node::List(list) => Some(list),
                        _ => None,
                    })
                    .flat_map(|list| list.children.iter())
                    .filter_map(|child| match child {
                        Node::ListItem(list_item) => Some(list_item),
                        _ => None,
                    })
                    .collect();
                let first = BlockBuilder::new()
                    .with_list_item(list_items[0].clone())
                    .with_file_path(
                        std::path::PathBuf::from(
                            "graph/pages/tests___parsing___blocks___tags_wikilinks.md",
                        )
                        .into(),
                    )
                    .get_content();
                first.unwrap()
            }

            #[test]
            fn test_get_wikilinks() {
                let content = get_content();
                let wikilinks = BlockBuilder::get_wikilinks(&content);
                assert_eq!(wikilinks, vec!["wikilink"]);
            }

            #[test]
            fn test_get_tags() {
                let content = get_content();
                let tags = BlockBuilder::get_tags(&content);
                assert_eq!(tags, vec!["multi word tag", "tag"]);
            }
        }
    }
}
