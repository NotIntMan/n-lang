use std::sync::Arc;
use std::collections::HashMap;
use helpers::{
    Path,
    PathBuf,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text {
    pub name: String,
    pub text: String,
}

impl Text {
    pub fn new<A: ToString, B: ToString>(name: A, text: B) -> Self {
        Text { name: name.to_string(), text: text.to_string() }
    }
}

pub trait TextSource {
    fn get_text(&self, path: Path) -> Option<Arc<Text>>;
}

#[derive(Clone)]
pub struct HashMapSource {
    map: HashMap<PathBuf, Arc<Text>>,
}

impl HashMapSource {
    pub fn new() -> Self {
        HashMapSource {
            map: HashMap::new(),
        }
    }
    pub fn simple_insert(&mut self, path: Path, name: &str, text: &str) {
        self.map.insert(
            PathBuf::from_path(path),
            Arc::new(Text {
                name: name.to_string(),
                text: text.to_string(),
            }),
        );
    }
}

impl TextSource for HashMapSource {
    fn get_text(&self, path: Path) -> Option<Arc<Text>> {
        for (text_path, text) in self.map.iter() {
            if text_path.as_path() == path {
                return Some(text.clone());
            }
        }
        None
    }
}
