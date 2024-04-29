use std::{collections::HashMap, path::Path};

use markdown::mdast::Node;
use regex::Regex;
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

    fn get_content(&self) -> Result<String, String> {
        let file_path = self.path.clone().ok_or("No path".to_string())?;
        let buf = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    fn get_top_text(ast: &Node) -> String {
        let top_text = ast
            .children()
            .unwrap()
            .iter()
            .flat_map(|node| node.children().unwrap())
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
        top_text
    }

    fn get_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    fn get_properties(top_text: &str) -> HashMap<String, String> {
        let mut properties = HashMap::new();
        for line in top_text.lines() {
            let split = line.split("::").map(|s| s.to_string()); // Convert iterator over &str to iterator over String
            if let [key, value] = split.collect::<Vec<String>>().as_slice() {
                match key.as_str() {
                    "title" => {}
                    "tags" => {}
                    _ => {
                        properties.insert(key.clone(), value.clone());
                    }
                }
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

    fn get_tags(top_text: &str, content: &str) -> Vec<String> {
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
        for line in top_text.lines() {
            let split = line.split("::").map(|s| s.to_string()); // Convert iterator over &str to iterator over String
            if let [key, value] = split.collect::<Vec<String>>().as_slice() {
                if key.as_str() == "tags" {
                    let tags_split: Vec<&str> = value.split(',').collect();
                    let trim_tags_split: Vec<&str> = tags_split.iter().map(|x| x.trim()).collect();
                    tags.extend(trim_tags_split.iter().map(|x| x.to_string()));
                }
            }
        }
        tags
    }

    fn get_title(path: &Path) -> String {
        let file_name = path
            .file_name()
            .expect("No file name")
            .to_str()
            .expect("No file name");
        file_name.replace(".md", "").replace("___", "/")
    }

    pub fn build(mut self) -> Result<File, String> {
        let ast = self.ast.take().ok_or("No AST".to_string())?;
        let path = self
            .path
            .clone()
            .ok_or("No path".to_string())?
            .to_string_lossy()
            .to_string();
        let top_text = Self::get_top_text(&ast);
        let content = self.get_content()?;
        let id = Self::get_id();
        let properties = Self::get_properties(&top_text);
        let wikilinks = Self::get_wikilinks(&content);
        let tags = Self::get_tags(&top_text, &content);
        let title = Self::get_title(
            self.path
                .take()
                .ok_or("No path".to_string())
                .as_ref()
                .unwrap(),
        );
        Ok(File {
            id,
            path,
            title,
            properties,
            wikilinks,
            tags,
        })
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
