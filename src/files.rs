//! Utilities for handling files and directories.

use glob::Pattern;
use markdown::mdast;
use walkdir::WalkDir;
use std::path::Path;

/// Walks a directory tree and yields files matching a glob pattern.
pub struct MdWalker {
    walker: walkdir::IntoIter,
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
    type Item = Result<(Box<Path>, mdast::Node), String>;

    /// Get the next file matching the pattern. Returns the markdown AST.
    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.walker.by_ref() {
            match entry {
                Ok(e) if self.pattern.matches_path(Path::new(e.path())) => {
                    let content = match std::fs::read_to_string(e.path()) {
                        Ok(content) => content,
                        Err(msg) => return Some(Err(msg.to_string())),
                    };
                    let ast = markdown::to_mdast(&content, &markdown::ParseOptions::default());
                    match ast {
                        Ok(ast) => return Some(Ok((e.path().to_path_buf().into_boxed_path(), ast))),
                        Err(msg) => return Some(Err(msg.to_string()))
                    }
                }
                Err(msg) => return Some(Err(msg.to_string())),
                Ok(_) => continue,
            }
        }
        None
    }
}