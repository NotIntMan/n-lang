use std::fmt;
use super::module::ModuleId;

#[derive(Debug)]
pub enum Item {
    DataType(),
    Alias(),
}

#[derive(Debug)]
pub enum ItemType {
    DataType,
    Alias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ItemId {
    pub module_id: ModuleId,
    pub item_id: usize,
}

#[derive(Debug)]
pub struct ItemContext {
    // requested dependencies
    // passed dependencies
    // thrown errors
    item_id: ItemId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticItemType {
    Field,
    DataType,
}

impl fmt::Display for SemanticItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &SemanticItemType::Field => write!(f, "field"),
            &SemanticItemType::DataType => write!(f, "data type"),
        }
    }
}
