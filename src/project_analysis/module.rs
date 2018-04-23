use std::sync::Arc;
use indexmap::IndexMap;
//use helpers::group::Group;
//use helpers::re_entrant_rw_lock::ReEntrantRWLock;
use helpers::resolve::Resolve;
use helpers::sync_ref::SyncRef;
use lexeme_scanner::{
//    ItemPosition,
Scanner,
};
use parser_basics::{
//    Identifier,
parse,
};
use syntax_parser::modules::{
    module,
    ModuleDefinitionItem,
    ModuleDefinitionItemAST,
};
//use syntax_parser::others::StaticPath;
use project_analysis::{
//    Item,
ModuleContext,
Text,
SemanticItemType,
SemanticError,
Project,
};
//use super::item::{
//    ItemBody,
//    ItemRef,
//};
//use super::error::SemanticError;
//
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedModule {
    text: Arc<Text>,
    source: &'static str,
    items: Vec<ModuleDefinitionItemAST<'static>>,
}

impl UnresolvedModule {
    pub fn new(text: Arc<Text>) -> Result<Self, Vec<SemanticError>> {
        let source = Box::leak(text.text.clone().into_boxed_str());
        let tokens = match Scanner::scan(source) {
            Ok(tokens) => tokens,
            Err(error) => return Err(vec![SemanticError::scanner_error(error)]),
        };
        let items = match parse(tokens.as_slice(), module) {
            Ok(items) => items,
            Err(errors) => return Err(
                errors.extract_into_vec()
                    .into_iter()
                    .map(|error| SemanticError::parser_error(error))
                    .collect()
            ),
        };
        Ok(UnresolvedModule {
            text,
            source,
            items,
        })
    }
    #[inline]
    pub fn from_text(text: Text) -> Result<Self, Vec<SemanticError>> {
        UnresolvedModule::new(Arc::new(text))
    }
}

impl Drop for UnresolvedModule {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.source as *const str as *mut str));
        }
    }
}

impl Resolve<SyncRef<Project>> for UnresolvedModule {
    type Result = Module;
    type Error = SemanticError;
    fn resolve(&self, ctx: &mut SyncRef<Project>) -> Result<Self::Result, Vec<Self::Error>> {
        let mut context = ModuleContext::new(ctx.clone());
        let items_vec = match self.items.resolve(&mut context) {
            Ok(items) => items,
            Err(mut errors) => {
                for error in errors.iter_mut() {
                    error.set_text(self.text.clone());
                }
                return Err(errors);
            }
        };
        let mut items = IndexMap::new();
        let mut errors = Vec::new();
        for (name, item) in items_vec {
            let pos = item.position;
            if items.insert(name.clone(), item).is_some() {
                errors.push(SemanticError::duplicate_definition(
                    pos,
                    name,
                    SemanticItemType::Definition,
                ))
            }
        }
        Ok(Module {
            items,
            project: ctx.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    items: IndexMap<String, ModuleDefinitionItem>,
    project: SyncRef<Project>,
}

//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct ModuleRef(pub Arc<ReEntrantRWLock<Module>>);
//
//impl Module {
//    pub fn try_parse(text: Arc<Text>) -> Result<ModuleRef, Group<SemanticError>> {
//        let tokens = match Scanner::scan(&text.text) {
//            Ok(tokens) => tokens,
//            Err(error) => return Err(Group::One(SemanticError::scanner_error(error))),
//        };
//        match parse(&tokens, module) {
//            Ok(items) => Ok(Module::from_def(text.clone(), items)),
//            Err(error_group) => Err(Group::new(
//                error_group.extract_into_vec().into_iter()
//                    .map(|item| SemanticError::parser_error(item))
//                    .collect()
//            )),
//        }
//    }
//    pub fn from_def(text: Arc<Text>, items: Vec<ModuleDefinitionItem>) -> ModuleRef {
//        let module_ref = ModuleRef(Arc::new(ReEntrantRWLock::new(Module {
//            text,
//            items: Vec::with_capacity(items.len()),
//            injected_dependencies: Vec::new(),
//        })));
//        {
//            let mut module = module_ref.0.write();
//            for item in items {
//                module.items.push(ItemRef::from_def(item))
//            }
//        }
//        module_ref
//    }
//    pub fn items(&self) -> &[ItemRef] {
//        &self.items
//    }
//    pub fn text(&self) -> Arc<Text> {
//        self.text.clone()
//    }
//}
//
//impl ModuleRef {
//    pub fn put_dependency(&self, path: StaticPath, dependency: &ModuleRef, errors: &mut Vec<SemanticError>) -> bool {
//        let module = self.0.read();
//        if module.injected_dependencies.contains(&path) {
//            return false;
//        }
//        println!("Putting {:?} into module {:?}", path.path, module.text.name);
//        for item in module.items.iter() {
//            match item.put_dependency(&path, dependency) {
//                Ok(()) => {}
//                Err(err) => errors.push(err),
//            }
//        }
//        {
//            let mut module = self.0.write();
//            module.injected_dependencies.push(path);
//        }
//        true
//    }
//    pub fn find_item(&self, name: &[Identifier]) -> Option<ItemRef> {
//        let module = self.0.read();
//        println!("Finding item {:?} in module {:?}", name, module.text.name);
//        if name.is_empty() {
//            return Some(ItemRef::from_body(ItemBody::ModuleReference { module: self.clone() }));
//        }
//        for item in module.items.iter() {
//            if let Some(item_ref) = item.find_item(name) {
//                return Some(item_ref);
//            }
//        }
//        None
//    }
//}
//
//#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
//pub struct ModuleId {
//    pub module_id: usize,
//}
//
