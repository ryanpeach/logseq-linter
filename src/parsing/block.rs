use markdown::mdast::{ListItem, Node};
use serde::{Deserialize, Serialize};

pub struct BlockBuilder {
    id: usize,
    list_item: Option<ListItem>,
    file_id: Option<usize>,
    parent_block_id: Option<usize>,
}

impl BlockBuilder {
    pub fn new() -> BlockBuilder {
        BlockBuilder {
            id: uuid::Uuid::new_v4().as_u128() as usize,
            list_item: None,
            file_id: None,
            parent_block_id: None,
        }
    }

    pub fn with_list_item(mut self, list_item: ListItem) -> BlockBuilder {
        self.list_item = Some(list_item);
        self
    }

    pub fn with_file_id(mut self, file_id: usize) -> BlockBuilder {
        self.file_id = Some(file_id);
        self
    }

    pub fn with_parent_block_id(mut self, parent_block_id: usize) -> BlockBuilder {
        self.parent_block_id = Some(parent_block_id);
        self
    }

    fn get_properties(&self) -> Vec<String> {
        let _list_item = self.list_item.as_ref().expect("No list item");
        todo!("Get the properties from the list item")
    }

    fn get_wikilinks(&self) -> Vec<String> {
        let _list_item = self.list_item.as_ref().expect("No list item");
        todo!("Get the wikilinks from the list item")
    }

    fn get_tags(&self) -> Vec<String> {
        let _list_item = self.list_item.as_ref().expect("No list item");
        todo!("Get the tags from the list item")
    }

    fn get_content(&self) -> String {
        let _list_item = self.list_item.as_ref().expect("No list item");
        todo!("Get the content from the list item")
    }

    pub fn build(mut self) -> Vec<Block> {
        let list_item = self.list_item.take().expect("No list item");
        let properties = self.get_properties();
        let wikilinks = self.get_wikilinks();
        let tags = self.get_tags();
        let content = self.get_content();
        let file_id = self.file_id.expect("No file id");
        let root = Block {
            id: self.id,
            content,
            file_id,
            properties,
            wikilinks,
            tags,
            parent_block_id: self.parent_block_id,
        };
        let mut blocks = vec![root];
        for child in list_item.children.iter() {
            if let Node::List(list) = child {
                for child in list.children.iter() {
                    if let Node::ListItem(list_item) = child {
                        let block = BlockBuilder::new()
                            .with_list_item(list_item.clone())
                            .with_file_id(file_id)
                            .with_parent_block_id(self.id)
                            .build();
                        blocks.extend(block);
                    }
                }
            }
        }
        blocks
    }
}

/// This is a logseq block, which is a markdown list element
#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    /// The index of the block in the list
    pub id: usize,
    /// The text content of the block divided into types
    pub content: String,
    /// The file this block belongs to
    pub file_id: usize,
    /// Parent block id
    pub parent_block_id: Option<usize>,
    /// The block properties
    pub properties: Vec<String>,
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
