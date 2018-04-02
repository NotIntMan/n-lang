use std::sync::Arc;
use super::project::ModulePathSlice;

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
    fn get_text(&self, path: &ModulePathSlice) -> Option<Arc<Text>>;
}
