use std::{collections::HashMap, path::Path};

use markdown::mdast::Node;
use serde::{Deserialize, Serialize};

pub struct FileBuilder {
    id: usize,
    path: Option<Box<Path>>,
    ast: Option<Node>,
}

impl FileBuilder {
    pub fn new() -> FileBuilder {
        FileBuilder {
            id: uuid::Uuid::new_v4().as_u128() as usize,
            path: None,
            ast: None,
        }
    }

    pub fn with_path(mut self, path: Box<Path>) -> FileBuilder {
        self.path = Some(path);
        self
    }

    pub fn with_ast(mut self, ast: Node) -> FileBuilder {
        self.ast = Some(ast);
        self
    }

    fn get_properties(&self) -> HashMap<String, String> {
        let _ast = self.ast.as_ref().expect("No AST");
        todo!("Get the properties from the AST")
    }

    fn get_wikilinks(&self) -> Vec<String> {
        let _ast = self.ast.as_ref().expect("No AST");
        todo!("Get the wikilinks from the AST")
    }

    fn get_tags(&self) -> Vec<String> {
        let _ast = self.ast.as_ref().expect("No AST");
        todo!("Get the tags from the AST")
    }

    fn get_title(&self) -> String {
        let _ast = self.ast.as_ref().expect("No AST");
        todo!("Get the title from the AST")
    }

    pub fn build(mut self) -> File {
        let _ast = self.ast.take().expect("No AST");
        let properties = self.get_properties();
        let wikilinks = self.get_wikilinks();
        let tags = self.get_tags();
        let title = self.get_title();
        File {
            id: self.id,
            path: self.path.expect("No path").to_string_lossy().to_string(),
            title,
            properties,
            wikilinks,
            tags,
        }
    }
}

/// This is a markdown file in logseq
#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    /// The id of the file
    pub id: usize,
    /// The path of the file
    pub path: String,
    /// The title of the file
    pub title: String,
    /// The page-properties of the file
    pub properties: HashMap<String, String>,
    /// wikilinks in the file
    pub wikilinks: Vec<String>,
    /// page tags
    pub tags: Vec<String>,
}

// impl File {
//     /// Get the tags from the AST
//     /// They are at the top of the file
//     /// Root { children: [Paragraph { children: [Text { value:
//     /// Before List
//     fn get_attributes(ast: &Node) -> HashMap<String, Vec<TypeEnum>> {
//         let mut attributes = HashMap::new();
//         let children = ast.children().expect("No children");
//         for child in children {
//             if let Node::Paragraph(paragraph) = child {
//                 for child in paragraph.children.iter() {
//                     if let Node::Text(text) = child {
//                         for line in text.value.lines() {
//                             let split = line.split("::");
//                             if let [key, values] = split.collect::<Vec<&str>>().as_slice() {
//                                 let key = key.trim();
//                                 let values = values.trim();
//                                 let values_split: Vec<&str> = values.split(',').collect();
//                                 let trim_values_split: Vec<&str> =
//                                     values_split.iter().map(|x| x.trim()).collect();
//                                 let mut type_enums = Vec::new();
//                                 for value in trim_values_split {
//                                     type_enums.push(TypeEnum::from_csv_item(value.to_string()));
//                                 }
//                                 attributes.insert(key.to_string(), type_enums);
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         attributes
//     }

//     /// Get the blocks from the AST
//     /// They are the list elements
//     /// Root { List { children: [ListItem { children: [Paragraph { children: [Text { value:
//     fn get_blocks(ast: &Node) -> Vec<Block> {
//         let mut blocks = Vec::new();
//         let children = ast.children().expect("No children");
//         for child in children.iter() {
//             if let Node::List(list) = child {
//                 for child in list.children.iter() {
//                     if let Node::ListItem(list_item) = child {
//                         let block = Block::new(blocks.len(), list_item);
//                         blocks.push(block);
//                     }
//                 }
//             }
//         }
//         blocks
//     }
// }
