use std::collections::HashMap;

use markdown::mdast::{ListItem, Node};
use serde::{Deserialize, Serialize};

pub struct BlockBuilder {
    list_item: Option<ListItem>,
    file_id: Option<String>,
    parent_block_id: Option<String>,
}

impl BlockBuilder {
    pub fn new() -> BlockBuilder {
        BlockBuilder {
            list_item: None,
            file_id: None,
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

    pub fn with_parent_block_id(mut self, parent_block_id: String) -> BlockBuilder {
        self.parent_block_id = Some(parent_block_id);
        self
    }

    fn get_content(&self) -> String {
        let _list_item = self.list_item.as_ref().expect("No list item");
        todo!("Get the content from the list item")
    }

    fn get_id(_content: &str) -> String {
        todo!("Get the id from the content");
    }

    fn get_properties(_content: &str) -> HashMap<String, String> {
        todo!("Get the properties from the content")
    }

    fn get_wikilinks(_content: &str) -> Vec<String> {
        todo!("Get the wikilinks from the content")
    }

    fn get_tags(_content: &str) -> Vec<String> {
        todo!("Get the tags from the content")
    }

    pub fn build(mut self) -> Vec<Block> {
        let list_item = self.list_item.take().expect("No list item");
        let content = self.get_content();
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
                            .build();
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
        blocks
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

// impl Block {
//     /// ListItem { children: [List { children: [ListItem { children: [Paragraph { children: [Text { value:
//     pub fn new(idx: usize, list_item: &ListItem) -> Block {
//         let mut content = Vec::new();
//         let mut sub_blocks = Vec::new();
//         for child in list_item.children.iter() {
//             if let Node::List(list) = child {
//                 for child in list.children.iter() {
//                     if let Node::ListItem(list_item) = child {
//                         let block = Block::new(sub_blocks.len(), list_item);
//                         sub_blocks.push(block);
//                     }
//                 }
//             } else if let Node::Paragraph(paragraph) = child {
//                 for child in paragraph.children.iter() {
//                     if let Node::Text(text) = child {
//                         content.extend(TypeEnum::from_text(text.value.clone()))
//                     }
//                 }
//             }
//         }
//         Block {
//             idx,
//             content,
//             sub_blocks,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    mod builder {
        use super::*;

        #[test]
        fn test_get_id() {
            todo!("Test get_id")
        }

        #[test]
        fn test_get_properties() {
            todo!("Test get_properties")
        }

        #[test]
        fn test_get_wikilinks() {
            todo!("Test get_wikilinks")
        }

        #[test]
        fn test_get_tags() {
            todo!("Test get_tags")
        }

        #[test]
        fn test_get_content() {
            todo!("Test get_content")
        }
    }

    mod block {
        use super::*;

        #[test]
        fn test_build() {
            todo!("Test build")
        }
    }
}
