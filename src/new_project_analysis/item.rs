use std::fmt;
use helpers::into_static::IntoStatic;
use syntax_parser::modules::{
    DataTypeDefinition,
    ExternalItemImport,
    ModuleDefinitionItem,
    ModuleDefinitionValue,
};
use super::module::ModuleId;

#[derive(Debug)]
pub enum Item {
    DataType(DataTypeDefinition<'static>),
    Import(ExternalItemImport<'static>),
}

impl Item {
    pub fn from_def(def: ModuleDefinitionItem) -> Self {
        let ModuleDefinitionItem {
            public: _,
            attributes: _,
            value,
        } = def.into_static();
        match value {
            ModuleDefinitionValue::DataType(def) => Item::DataType(def),
            ModuleDefinitionValue::Import(def) => Item::Import(def),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub enum ItemType {
    DataType,
    Import,
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
