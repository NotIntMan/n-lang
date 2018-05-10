use std::fmt;
//use std::sync::Arc;
////use helpers::IntoStatic;
//use helpers::ReEntrantRWLock;
use helpers::{
    Path,
    SyncRef,
};
//use lexeme_scanner::ItemPosition;
//use parser_basics::{
//    Identifier,
//    StaticIdentifier,
//};
use language::{
    DataTypeDefinition,
    FunctionDefinition,
//    ExternalItemImport,
//    ModuleDefinitionItem,
////    ModuleDefinitionValue,
//    TableDefinition,
};
//use language::others::StaticPath;
//use super::resolve::{
//    SemanticResolve,
//    ResolveContext,
//};
//use super::module::ModuleRef;
//use super::error::SemanticError;
//
use project_analysis::Module;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    body: ItemBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemBody {
    DataType {
        def: DataTypeDefinition,
    },
    //    ImportDefinition {
//        def: ExternalItemImport<'static>
//    },
//    ImportItem {
//        name: StaticIdentifier,
//        original_name: StaticIdentifier,
//        item: ItemRef,
//    },
    ModuleReference {
        module: SyncRef<Module>,
    },
    //    Table {
//        def: TableDefinition<'static>,
//        primary_key: Result<ItemRef, SemanticError>,
//    },
    Function {
        def: FunctionDefinition,
    },
}

impl Item {
    #[inline]
    pub fn data_type(def: DataTypeDefinition) -> Self {
        Item { body: ItemBody::DataType { def } }
    }
    #[inline]
    pub fn module_ref(module: SyncRef<Module>) -> Self {
        Item { body: ItemBody::ModuleReference { module } }
    }
    #[inline]
    pub fn function(def: FunctionDefinition) -> Self {
        Item { body: ItemBody::Function { def } }
    }
    #[inline]
    pub fn get_type(&self) -> SemanticItemType {
        match &self.body {
            &ItemBody::DataType { def: _ } => SemanticItemType::DataType,
//            &ItemBody::ImportDefinition { def: _ } => SemanticItemType::UnresolvedImport,
//            &ItemBody::ImportItem { name: _, original_name: _, ref item } => item.get_type(),
            &ItemBody::ModuleReference { module: _ } => SemanticItemType::Module,
//            &ItemBody::Table { def: _, primary_key: _ } => SemanticItemType::Table,
            &ItemBody::Function { def: _ } => SemanticItemType::Function,
        }
    }
    #[inline]
    pub fn get_data_type(&self) -> Option<&DataTypeDefinition> {
        match &self.body {
            &ItemBody::DataType { ref def } => Some(def),
            _ => None,
        }
    }
    #[inline]
    pub fn get_module_ref(&self) -> Option<&SyncRef<Module>> {
        match &self.body {
            &ItemBody::ModuleReference { ref module } => Some(module),
            _ => None,
        }
    }
    #[inline]
    pub fn get_function(&self) -> Option<&FunctionDefinition> {
        match &self.body {
            &ItemBody::Function { ref def } => Some(def),
            _ => None,
        }
    }
}

impl SyncRef<Item> {
    pub fn get_item(&self, path: Path, search_route: &mut Vec<SyncRef<Module>>) -> Option<Self> {
        if path.is_empty() {
            return Some(self.clone());
        }
        let item = self.read();
        match &item.body {
            &ItemBody::DataType { def: _ } => {}
            &ItemBody::ModuleReference { ref module } => {
                return module.get_item(path, search_route);
            }
            &ItemBody::Function { def: _ } => {}
        }
        None
    }
    #[inline]
    pub fn get_type(&self) -> SemanticItemType {
        self.read().get_type()
    }
}

//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct ItemRef(pub Arc<ReEntrantRWLock<Item>>);
//
//impl ItemRef {
//    pub fn from_def(_def: ModuleDefinitionItem) -> Self {
//        unimplemented!();
////        let ModuleDefinitionItem {
////            public: _,
////            attributes: _,
////            value,
////        } = def.into_static();
////        let body = match value {
////            ModuleDefinitionValue::DataType(def) => {
////                ItemBody::DataType { def }
////            }
////            ModuleDefinitionValue::Import(def) => ItemBody::ImportDefinition { def },
////            ModuleDefinitionValue::Table(def) => {
////                let primary_key = def.make_primary_key();
////                ItemBody::Table { def, primary_key }
////            }
////            ModuleDefinitionValue::Function(_def) => {
////                unimplemented!()
////            }
////            _ => unimplemented!(),
////        };
////        ItemRef::from_body(body)
//    }
//    #[inline]
//    pub fn from_body(body: ItemBody) -> Self {
//        let item = Item {
//            is_resolved: false,
//            body,
//        };
//        ItemRef(Arc::new(ReEntrantRWLock::new(item)))
//    }
//    pub fn find_item(&self, name: &[Identifier]) -> Option<ItemRef> {
//        let item = self.0.read();
//        println!("Finding item {:?} in item {:?}", name, *item);
//        match &item.body {
//            &ItemBody::DataType { ref def } => {
//                if name.len() == 1
//                    && name[0] == def.name {
//                    return Some(self.clone());
//                }
//            }
//            &ItemBody::ImportDefinition { def: _ } => {}
//            &ItemBody::ImportItem { name: ref import_name, ref original_name, ref item } => {
//                if (name.len() > 0)
//                    && name[0] == *import_name {
//                    return match item.get_module(ItemPosition::default()) {
//                        Ok(module) => module.find_item(&name[1..]),
//                        Err(_) => {
//                            let mut name_inside_import = Vec::with_capacity(name.len());
//                            name_inside_import.push(original_name.clone());
//                            for name_item in &name[1..] {
//                                name_inside_import.push(name_item.clone());
//                            }
//                            item.find_item(name_inside_import.as_slice())
//                        }
//                    };
//                }
//            }
//            &ItemBody::ModuleReference { ref module } => {
//                return module.find_item(name);
//            }
//            &ItemBody::Table { ref def, ref primary_key } => {
//                match name.len() {
//                    1 => if name[0] == def.name {
//                        return Some(self.clone());
//                    }
//                    2 => if name[0] == def.name
//                        && name[1].get_text() == "primary_key" {
//                        if let &Ok(ref item) = primary_key {
//                            return Some(item.clone());
//                        }
//                    }
//                    _ => {}
//                }
//            }
//        }
//        None
//    }
//    pub fn get_type(&self) -> SemanticItemType {
//        let item = self.0.read();
//        match &item.body {
//            &ItemBody::DataType { def: _ } => SemanticItemType::DataType,
//            &ItemBody::ImportDefinition { def: _ } => SemanticItemType::UnresolvedImport,
//            &ItemBody::ImportItem { name: _, original_name: _, ref item } => item.get_type(),
//            &ItemBody::ModuleReference { module: _ } => SemanticItemType::Module,
//            &ItemBody::Table { def: _, primary_key: _ } => SemanticItemType::Table,
//        }
//    }
//    pub fn assert_type(&self, item_type: ItemType, pos: ItemPosition) -> Result<(), SemanticError> {
//        let item = self.0.read();
//        let expected = item_type.into_semantic();
//        let got = self.get_type();
//        println!("Asserting item type ({} == {}) of {:?}", expected, got, *item);
//        if expected == got {
//            Ok(())
//        } else {
//            Err(SemanticError::expected_item_of_another_type(pos, expected, got))
//        }
//    }
//    //    pub fn get_data_type(&self, pos: ItemPosition) -> Result<DataTypeDefinition<'static>, SemanticError> {
////        let item = self.0.read();
////        match &item.body {
////            &ItemBody::DataType(ref def) => Ok(def.clone()),
////            _ => Err(SemanticError::expected_item_of_another_type(pos, SemanticItemType::DataType)),
////        }
////    }
//    pub fn get_module(&self, pos: ItemPosition) -> Result<ModuleRef, SemanticError> {
//        let item = self.0.read();
//        match &item.body {
//            &ItemBody::ModuleReference { ref module } => Ok(module.clone()),
//            _ => Err(SemanticError::expected_item_of_another_type(pos, SemanticItemType::Module, self.get_type())),
//        }
//    }
//    pub fn put_dependency(&self, dependency: &StaticPath, module: &ModuleRef) -> Result<(), SemanticError> {
//        println!("Putting {:?} into item {:?}", dependency.path, self.0);
//        let mut new_body = None;
//        {
//            let item = self.0.read();
//            match &item.body {
//                &ItemBody::ImportDefinition { ref def } =>
//                    if let Some(body) = def.try_put_dependency(dependency, module)? {
//                        new_body = Some(body);
//                    }
//                _ => {}
//            }
//        }
//        if let Some(new_body) = new_body {
//            self.0.write().body = new_body;
//        }
//        Ok(())
//    }
//}
//
//impl SemanticResolve for Item {
//    #[inline]
//    fn is_resolved(&self, _context: &ResolveContext) -> bool {
//        self.is_resolved
//    }
//    fn try_resolve(&mut self, context: &mut ResolveContext) {
//        let mut new_body = None;
//        match &mut self.body {
//            &mut ItemBody::DataType { def: _ } => {
//                unimplemented!()
//                // def.body.try_resolve(context);
//                // self.is_resolved = def.body.is_resolved(context);
//            }
//            &mut ItemBody::ImportDefinition { ref mut def } => {
//                if let Some(body) = def.try_semantic_resolve(context) {
//                    self.is_resolved = true;
//                    new_body = Some(body);
//                }
//            }
//            &mut ItemBody::ImportItem { name: _, original_name: _, item: _ } => self.is_resolved = true,
//            &mut ItemBody::ModuleReference { module: _ } => self.is_resolved = true,
//            &mut ItemBody::Table { def: _, primary_key: _ } => {
//                unimplemented!()
////                def.try_resolve(context);
////                self.is_resolved = if def.is_resolved(context) {
////                    match primary_key {
////                        &mut Ok(ref item) => {
////                            let mut item = item.0.write();
////                            item.try_resolve(context);
////                            item.is_resolved(context)
////                        }
////                        &mut Err(ref err) => {
////                            context.throw_error(err.clone());
////                            false
////                        }
////                    }
////                } else {
////                    false
////                };
//            }
//        }
//        if let Some(new_body) = new_body {
//            self.body = new_body;
//        }
//    }
//}
//
//#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//pub enum ItemType {
//    DataType,
//    Module,
//    Table,
//}
//
//impl ItemType {
//    pub fn into_semantic(self) -> SemanticItemType {
//        match self {
//            ItemType::DataType => SemanticItemType::DataType,
//            ItemType::Module => SemanticItemType::Module,
//            ItemType::Table => SemanticItemType::Table,
//        }
//    }
//}
//
//#[derive(Debug)]
//pub struct ItemContext {
//    // requested dependencies
//    // passed dependencies
//    // thrown errors
////    item_id: ItemId,
//}
//
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticItemType {
    Definition,
    Field,
    DataType,
    Module,
    Table,
    Variable,
    Function,
}

impl SemanticItemType {
    pub fn get_description(&self) -> &'static str {
        match self {
            &SemanticItemType::Definition => "definition",
            &SemanticItemType::Field => "field",
            &SemanticItemType::DataType => "data type",
            &SemanticItemType::Module => "module",
            &SemanticItemType::Table => "table",
            &SemanticItemType::Variable => "variable",
            &SemanticItemType::Function => "function",
        }
    }
}

impl fmt::Display for SemanticItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_description())
    }
}
