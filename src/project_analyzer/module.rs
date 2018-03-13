use helpers::storage::{
    SourceStorage,
    Storage,
};
use super::name_storage::{
    NameIndex,
    NameStorage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    names: NameStorage,
}

impl Module {
    pub fn store_name(&mut self, name: &str) -> NameIndex { self.names.store_element(name) }
}

impl SourceStorage<NameIndex> for Module {
    type Element = str;
    fn get_element(&self, index: NameIndex) -> Option<&str> { self.names.get_element(index) }
}
