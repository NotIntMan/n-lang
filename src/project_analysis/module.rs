use std::sync::Arc;
use helpers::group::Group;
use helpers::re_entrant_rw_lock::ReEntrantRWLock;
use lexeme_scanner::Scanner;
use parser_basics::{
    Identifier,
    parse,
};
use syntax_parser::modules::{
    module,
    ModuleDefinitionItem,
};
use syntax_parser::others::StaticPath;
use super::source::Text;
use super::item::{
    ItemBody,
    ItemRef,
};
use super::error::SemanticError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    text: Arc<Text>,
    items: Vec<ItemRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleRef(pub Arc<ReEntrantRWLock<Module>>);

impl Module {
    pub fn try_parse(text: Arc<Text>) -> Result<ModuleRef, Group<SemanticError>> {
        let tokens = match Scanner::scan(&text.text) {
            Ok(tokens) => tokens,
            Err(error) => return Err(Group::One(SemanticError::scanner_error(error))),
        };
        match parse(&tokens, module) {
            Ok(items) => Ok(Module::from_def(text.clone(), items)),
            Err(error_group) => Err(Group::new(
                error_group.extract_into_vec().into_iter()
                    .map(|item| SemanticError::parser_error(item))
                    .collect()
            )),
        }
    }
    pub fn from_def(text: Arc<Text>, items: Vec<ModuleDefinitionItem>) -> ModuleRef {
        let module_ref = ModuleRef(Arc::new(ReEntrantRWLock::new(Module {
            text,
            items: Vec::with_capacity(items.len()),
        })));
        {
            let mut module = module_ref.0.write();
            for item in items {
                module.items.push(ItemRef::from_def(item))
            }
        }
        module_ref
    }
    pub fn items(&self) -> &[ItemRef] {
        &self.items
    }
    pub fn text(&self) -> Arc<Text> {
        self.text.clone()
    }
}

impl ModuleRef {
    pub fn put_dependency(&self, path: StaticPath, dependency: &ModuleRef, errors: &mut Vec<SemanticError>) {
        let module = self.0.read();
        println!("Putting {:?} into module {:?}", path.path, module.text.name);
        for item in module.items.iter() {
            match item.put_dependency(&path, dependency) {
                Ok(()) => {}
                Err(err) => errors.push(err),
            }
        }
    }
    pub fn find_item(&self, name: &[Identifier]) -> Option<ItemRef> {
        let module = self.0.read();
        println!("Finding item {:?} in module {:?}", name, module.text.name);
        if name.is_empty() {
            return Some(ItemRef::from_body(ItemBody::ModuleReference(self.clone())));
        }
        for item in module.items.iter() {
            if let Some(item_ref) = item.find_item(name) {
                return Some(item_ref);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ModuleId {
    pub module_id: usize,
}

