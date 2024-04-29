use std::{collections::HashMap, path::Path};

use markdown::mdast::Node;
use serde::{Deserialize, Serialize};

pub struct FileBuilder {
    path: Option<Box<Path>>,
    ast: Option<Node>,
}

impl FileBuilder {
    pub fn new() -> FileBuilder {
        FileBuilder {
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

    fn get_id(_ast: &Node) -> String {
        todo!("Get the id from the content");
    }

    fn get_properties(_ast: &Node) -> HashMap<String, String> {
        todo!("Get the properties from the AST")
    }

    fn get_wikilinks(_ast: &Node) -> Vec<String> {
        todo!("Get the wikilinks from the AST")
    }

    fn get_tags(_ast: &Node) -> Vec<String> {
        todo!("Get the tags from the AST")
    }

    fn get_title(_ast: &Node) -> String {
        todo!("Get the title from the AST")
    }

    pub fn build(mut self) -> File {
        let ast = self.ast.take().expect("No AST");
        let path = self
            .path
            .take()
            .expect("No path")
            .file_name()
            .expect("No file name")
            .to_str()
            .expect("No file name")
            .to_string();
        let id = Self::get_id(&ast);
        let properties = Self::get_properties(&ast);
        let wikilinks = Self::get_wikilinks(&ast);
        let tags = Self::get_tags(&ast);
        let title = Self::get_title(&ast);
        File {
            id,
            path,
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
    pub id: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    mod builder {
        use super::*;

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
        fn test_get_title() {
            todo!("Test get_title")
        }
    }

    mod file {
        use super::*;

        #[test]
        fn test_build() {
            todo!("Test build")
        }
    }
}
