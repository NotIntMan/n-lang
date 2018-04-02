use std::fmt;
use std::sync::Arc;
use helpers::into_static::IntoStatic;
use helpers::loud_rw_lock::LoudRwLock;
use parser_basics::Identifier;
use syntax_parser::modules::{
    DataTypeDefinition,
    ExternalItemImport,
    ModuleDefinitionItem,
    ModuleDefinitionValue,
};
use super::resolve::{
    SemanticResolve,
    ResolveContext,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    is_resolved: bool,
    body: ItemBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemBody {
    DataType(DataTypeDefinition<'static>),
    Import(ExternalItemImport<'static>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRef(pub Arc<LoudRwLock<Item>>);

impl Item {
    pub fn from_def(def: ModuleDefinitionItem) -> ItemRef {
        let ModuleDefinitionItem {
            public: _,
            attributes: _,
            value,
        } = def.into_static();
        let body = match value {
            ModuleDefinitionValue::DataType(def) => ItemBody::DataType(def),
            ModuleDefinitionValue::Import(def) => ItemBody::Import(def),
            _ => unimplemented!(),
        };
        let item = Item {
            is_resolved: false,
            body,
        };
        ItemRef(Arc::new(LoudRwLock::new(item, "Item was poisoned!")))
    }
}

impl ItemRef {
    pub fn find_item(&self, item_type: ItemType, name: &[Identifier]) -> Option<ItemRef> {
        let item = self.0.read();
        match &item.body {
            &ItemBody::DataType(ref def) => {
                if (item_type == ItemType::DataType)
                    && name.len() == 1
                    && name[0] == def.name {
                    Some(self.clone())
                } else {
                    None
                }
            }
            &ItemBody::Import(ref _def) => {
                unimplemented!()
            }
        }
    }
}

impl SemanticResolve for Item {
    #[inline]
    fn is_resolved(&self, _context: &ResolveContext) -> bool {
        self.is_resolved
    }
    fn try_resolve(&mut self, context: &mut ResolveContext) {
        match &mut self.body {
            &mut ItemBody::DataType(ref mut def) => def.body.try_resolve(context),
            &mut ItemBody::Import(ref mut _def) => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    DataType,
    Import,
}

#[derive(Debug)]
pub struct ItemContext {
    // requested dependencies
    // passed dependencies
    // thrown errors
//    item_id: ItemId,
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
