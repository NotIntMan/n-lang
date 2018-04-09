use std::sync::Arc;
use helpers::group::Group;
use helpers::loud_rw_lock::LoudRwLock;
use lexeme_scanner::Scanner;
use parser_basics::{
    Identifier,
    parse,
    StaticIdentifier,
};
use syntax_parser::modules::{
    module,
    ModuleDefinitionItem,
};
use syntax_parser::others::StaticPath;
use super::source::{
    Text,
    TextSource,
};
use super::item::{
    Item,
    ItemRef,
    ItemType,
};
use super::error::SemanticError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    text: Arc<Text>,
    items: Vec<ItemRef>,
}

pub type ModuleRef = Arc<LoudRwLock<Module>>;

impl Module {
    fn try_load_particular<S: TextSource>(source: &S, path: StaticPath) -> Result<(Arc<Text>, Vec<StaticIdentifier>), Group<SemanticError>> {
        let path_len = path.path.len();
        for i in 0..=path_len {
            let module_path_len = path_len - i;
            let module_path = &path.path[..module_path_len];
            if let Some(text) = source.get_text(module_path) {
                let rest_path = path.path[module_path_len..].to_vec();
                return Ok((text, rest_path));
            }
        }
        Err(Group::One(SemanticError::unresolved_item(
            path.pos,
            path.path,
        )))
    }
    pub fn try_load<S: TextSource>(source: &S, path: StaticPath) -> Result<(ModuleRef, Vec<StaticIdentifier>), Group<SemanticError>> {
        let (text, rest_path) = Module::try_load_particular(source, path)?;
        let tokens = match Scanner::scan(&text.text) {
            Ok(tokens) => tokens,
            Err(error) => return Err(Group::One(SemanticError::scanner_error(error))),
        };
        let module = match parse(&tokens, module) {
            Ok(items) => Module::from_def(text.clone(), items),
            Err(error_group) => return Err(Group::new(
                error_group.extract_into_vec().into_iter()
                    .map(|item| SemanticError::parser_error(item))
                    .collect()
            )),
        };
        let module_ref = Arc::new(LoudRwLock::new(module, "Module was poisoned!"));
        Ok((module_ref, rest_path))
    }
    pub fn from_def(text: Arc<Text>, items: Vec<ModuleDefinitionItem>) -> Self {
        Module {
            text,
            items: items.into_iter()
                .map(|def| Item::from_def(def))
                .collect(),
        }
    }
    pub fn find_item(&self, item_type: ItemType, name: &[Identifier]) -> Option<ItemRef> {
        println!("Finding in module item {:?}", name);
        for item in self.items.iter() {
            if let Some(item_ref) = item.find_item(item_type, name) {
                return Some(item_ref);
            }
        }
        None
    }
    pub fn items(&self) -> &[ItemRef] {
        &self.items
    }
    pub fn text(&self) -> Arc<Text> {
        self.text.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ModuleId {
    pub module_id: usize,
}

