//! These define the data structures for logseq files
use std::path::Path;
use markdown::mdast::Node;
use serde::{Deserialize, Serialize};

/// This is a markdown file in logseq
#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub id: usize,
    pub title: String,
    pub path: Box<Path>,
    pub blocks: Vec<Block>,
}

/// These are the relevant types in a logseq block for our purposes
#[derive(Serialize, Deserialize, Debug)]
pub enum TypeEnum {
    Text(String),
    Backlink(String),
    Tag(String),
}

/// This is a logseq block, which is a markdown list element
#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub idx: usize,
    pub content: Vec<TypeEnum>,
    pub sub_blocks: Vec<Block>,
}

impl File {
    /// Create a new `File` with a given path.
    pub fn new(id: usize, ast: Node, path: Box<Path>) -> File {
        println!("{:?}", ast);
        File {
            id,
            path,
            title: "title".to_string(),
            blocks: vec![],
        }
    }
}