//! These define the data structures for logseq files
use markdown::mdast::{ListItem, Node};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// These are the relevant types in a logseq block for our purposes
#[derive(Serialize, Deserialize, Debug)]
pub enum TypeEnum {
    Text(String),
    Backlink(String),
    Tag(String),
}

impl TypeEnum {
    pub fn from_csv_item(item: String) -> TypeEnum {
        if item.starts_with('#') {
            TypeEnum::Tag(item)
        } else if item.starts_with("[[") && item.ends_with("]]") {
            TypeEnum::Backlink(item)
        } else {
            TypeEnum::Text(item)
        }
    }

    pub fn from_text(text: String) -> Vec<TypeEnum> {
        let mut type_enums = Vec::new();
        const BACKLINK_REGEX: &str = r"\[\[.*?\]\]";
        const TAG_REGEX1: &str = r"#\w+";
        const TAG_REGEX2: &str = r"#\[\[.*?\]\]";
        let combined_regex: String = format!("{}|{}|{}", BACKLINK_REGEX, TAG_REGEX1, TAG_REGEX2);
        let backlink_re = Regex::new(BACKLINK_REGEX).unwrap();
        let tag_re1 = Regex::new(TAG_REGEX1).unwrap();
        let tag_re2 = Regex::new(TAG_REGEX2).unwrap();
        let combined_re = Regex::new(&combined_regex).unwrap();
        for cap in combined_re.split(&text) {
            if backlink_re.is_match(cap) {
                type_enums.push(TypeEnum::Backlink(cap.to_string()));
            } else if tag_re1.is_match(cap) || tag_re2.is_match(cap) {
                type_enums.push(TypeEnum::Tag(cap.to_string()));
            } else {
                type_enums.push(TypeEnum::Text(cap.to_string()));
            }
        }
        type_enums
    }
}

/// This is a logseq block, which is a markdown list element
#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub idx: usize,
    pub content: Vec<TypeEnum>,
    pub sub_blocks: Vec<Block>,
}

impl Block {
    /// ListItem { children: [List { children: [ListItem { children: [Paragraph { children: [Text { value:
    pub fn new(idx: usize, list_item: &ListItem) -> Block {
        let mut content = Vec::new();
        let mut sub_blocks = Vec::new();
        for child in list_item.children.iter() {
            if let Node::List(list) = child {
                for child in list.children.iter() {
                    if let Node::ListItem(list_item) = child {
                        let block = Block::new(sub_blocks.len(), list_item);
                        sub_blocks.push(block);
                    }
                }
            } else if let Node::Paragraph(paragraph) = child {
                for child in paragraph.children.iter() {
                    if let Node::Text(text) = child {
                        content.extend(TypeEnum::from_text(text.value.clone()))
                    }
                }
            }
        }
        Block {
            idx,
            content,
            sub_blocks,
        }
    }
}

/// This is a markdown file in logseq
#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub id: usize,
    pub path: Box<Path>,
    pub tags: HashMap<String, Vec<TypeEnum>>,
    pub blocks: Vec<Block>,
}

impl File {
    /// Create a new `File` with a given path.
    pub fn new(id: usize, ast: &Node, path: Box<Path>) -> File {
        File {
            id,
            path,
            tags: File::get_tags(ast),
            blocks: File::get_blocks(ast),
        }
    }

    /// Get the tags from the AST
    /// They are at the top of the file
    /// Root { children: [Paragraph { children: [Text { value:
    /// Before List
    fn get_tags(ast: &Node) -> HashMap<String, Vec<TypeEnum>> {
        let mut tags = HashMap::new();
        let children = ast.children().expect("No children");
        for child in children {
            if let Node::Paragraph(paragraph) = child {
                for child in paragraph.children.iter() {
                    if let Node::Text(text) = child {
                        for line in text.value.lines() {
                            let split = line.split("::");
                            if let [key, values] = split.collect::<Vec<&str>>().as_slice() {
                                let key = key.trim();
                                let values = values.trim();
                                let values_split: Vec<&str> = values.split(',').collect();
                                let trim_values_split: Vec<&str> =
                                    values_split.iter().map(|x| x.trim()).collect();
                                let mut type_enums = Vec::new();
                                for value in trim_values_split {
                                    type_enums.push(TypeEnum::from_csv_item(value.to_string()));
                                }
                                tags.insert(key.to_string(), type_enums);
                            }
                        }
                    }
                }
            }
        }
        tags
    }

    /// Get the blocks from the AST
    /// They are the list elements
    /// Root { List { children: [ListItem { children: [Paragraph { children: [Text { value:
    fn get_blocks(ast: &Node) -> Vec<Block> {
        let mut blocks = Vec::new();
        let children = ast.children().expect("No children");
        for child in children.iter() {
            if let Node::List(list) = child {
                for child in list.children.iter() {
                    if let Node::ListItem(list_item) = child {
                        let block = Block::new(blocks.len(), list_item);
                        blocks.push(block);
                    }
                }
            }
        }
        blocks
    }
}
