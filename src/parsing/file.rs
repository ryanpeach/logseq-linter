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
            .flat_map(|paragraph| paragraph.children().unwrap())
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
                        properties.insert(key.trim().to_string(), value.trim().to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    mod builder {
        use super::*;

        fn get_content() -> File {
            let content =
                std::fs::read_to_string("graph/pages/tests___parsing___files___basic.md").unwrap();
            let ast = markdown::to_mdast(&content, &markdown::ParseOptions::default()).unwrap();

            
            FileBuilder::new()
                .with_path(
                    std::path::PathBuf::from("graph/pages/tests___parsing___files___basic.md")
                        .into(),
                )
                .with_ast(ast)
                .build()
                .unwrap()
        }

        #[test]
        fn test_get_top_text() {
            let content =
                std::fs::read_to_string("graph/pages/tests___parsing___files___basic.md").unwrap();
            let ast = markdown::to_mdast(&content, &markdown::ParseOptions::default()).unwrap();
            let top_text = FileBuilder::get_top_text(&ast);
            assert_eq!(top_text, "tags:: foo, bar\nfoo:: bar");
        }

        #[test]
        fn test_get_properties() {
            let file = get_content();
            let properties = file.properties;
            assert_eq!(properties.get("foo"), Some(&"bar".to_string()));
            assert_eq!(properties.get("tags"), None);
        }

        #[test]
        fn test_get_wikilinks() {
            let file = get_content();
            let wikilinks = file.wikilinks;
            assert_eq!(wikilinks, vec!["wikilink"]);
        }

        #[test]
        fn test_get_tags() {
            let file = get_content();
            let tags = file.tags;
            assert_eq!(tags, vec!["foo", "bar", "tag", "multi word tag"]);
        }

        #[test]
        fn test_get_title() {
            let file = get_content();
            let title = file.title;
            assert_eq!(title, "tests/parsing/files/basic");
        }
    }
}
