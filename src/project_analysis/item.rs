use std::fmt;
use std::sync::Arc;
use helpers::into_static::IntoStatic;
use helpers::re_entrant_rw_lock::ReEntrantRWLock;
use parser_basics::{
    Identifier,
    StaticIdentifier,
};
use syntax_parser::modules::{
    DataTypeDefinition,
    ExternalItemImport,
    ExternalItemTail,
    ModuleDefinitionItem,
    ModuleDefinitionValue,
};
use syntax_parser::others::StaticPath;
use super::resolve::{
    SemanticResolve,
    ResolveContext,
};
use super::module::ModuleRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    is_resolved: bool,
    body: ItemBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemBody {
    DataType(DataTypeDefinition<'static>),
    ImportDefinition(ExternalItemImport<'static>),
    ImportItem(StaticIdentifier, ItemRef),
//    ImportModule(StaticPath, ModuleRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRef(pub Arc<ReEntrantRWLock<Item>>);

impl Item {
    pub fn from_def(def: ModuleDefinitionItem) -> ItemRef {
        let ModuleDefinitionItem {
            public: _,
            attributes: _,
            value,
        } = def.into_static();
        let body = match value {
            ModuleDefinitionValue::DataType(def) => {
                ItemBody::DataType(def)
            }
            ModuleDefinitionValue::Import(def) => ItemBody::ImportDefinition(def),
            _ => unimplemented!(),
        };
        let item = Item {
            is_resolved: false,
            body,
        };
        ItemRef(Arc::new(ReEntrantRWLock::new(item)))
    }
}

impl ItemRef {
    pub fn find_item(&self, item_type: ItemType, name: &[Identifier]) -> Option<ItemRef> {
        println!("Finding in item reference item {:?}", name);
        let item = self.0.read();
        match &item.body {
            &ItemBody::DataType(ref def) => {
                if ((item_type == ItemType::Unknown) || (item_type == ItemType::DataType))
                    && name.len() == 1
                    && name[0] == def.name {
                    Some(self.clone())
                } else {
                    None
                }
            }
            &ItemBody::ImportDefinition(_) => None,
            &ItemBody::ImportItem(ref import_name, ref item) => {
                if (name.len() > 0)
                    && name[0] == *import_name {
                    Some((*item).clone())
                } else {
                    None
                }
            }
//            _ => unimplemented!()
        }
    }
    pub fn put_dependency(&self, dependency: &StaticPath, module: &ModuleRef) {
        println!("Putting {:?} into item {:?}", dependency.path, self.0);
        let mut new_body = None;
        {
            let item = self.0.read();
            match &item.body {
                &ItemBody::ImportDefinition(ExternalItemImport { ref path, tail: ExternalItemTail::None }) => {
                    let dependency_len = dependency.path.len();
                    println!("Comparing paths (begin of {:?} and {:?}", dependency.path, path.path);
                    if (path.path.len() >= dependency_len)
                        &&
                        (dependency.path.as_slice() == &path.path[..dependency_len]) {
                        println!("Begin of dependency's path is equal to import's path. Trying to find item in dependency.");
                        let item_path = &path.path[dependency_len..];
                        if item_path.is_empty() {
                            panic!("Module import is not implemented yet");
                        }
                        let module = module.read();
                        match module.find_item(ItemType::Unknown, item_path) {
                            Some(item) => {
                                println!("Item found, putting {:?}", item);
                                let name = path.path.last()
                                    .expect("Path should not be empty!")
                                    .clone()
                                ;
                                new_body = Some(ItemBody::ImportItem(name, item));
                            }
                            None => return,
                        }
                    }
                }
                &ItemBody::ImportDefinition(ExternalItemImport { path: ref _path, tail: ExternalItemTail::Alias(_) }) => unimplemented!(),
                &ItemBody::ImportDefinition(ExternalItemImport { path: ref _path, tail: ExternalItemTail::Asterisk }) => unimplemented!(),
                _ => {}
            }
        }
        if let Some(new_body) = new_body {
            self.0.write().body = new_body;
        }
    }
}

impl SemanticResolve for Item {
    #[inline]
    fn is_resolved(&self, _context: &ResolveContext) -> bool {
        self.is_resolved
    }
    fn try_resolve(&mut self, context: &mut ResolveContext) {
        let mut new_body = None;
        match &mut self.body {
            &mut ItemBody::DataType(ref mut def) => {
                def.body.try_resolve(context);
                self.is_resolved = def.body.is_resolved(context);
            }
            &mut ItemBody::ImportDefinition(ref mut def) => {
                if let Some(body) = def.try_semantic_resolve(context) {
                    self.is_resolved = true;
                    new_body = Some(body);
                }
            }
            &mut ItemBody::ImportItem(_, _) => self.is_resolved = true,
            _ => unimplemented!(),
        }
        if let Some(new_body) = new_body {
            self.body = new_body;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Unknown,
    DataType,
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
