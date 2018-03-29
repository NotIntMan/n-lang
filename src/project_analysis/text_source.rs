use std::fmt::Debug;
use std::sync::Arc;
use std::fmt;
use helpers::group::Group;
use helpers::into_static::IntoStatic;
use lexeme_scanner::Scanner;
use parser_basics::{
    parse,
    StaticIdentifier,
};
use syntax_parser::modules::{
    module,
    ModuleDefinitionItem,
};
use super::error::SemanticError;

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
    fn get_text(&mut self, path: &[StaticIdentifier]) -> Option<Arc<Text>>;

    fn try_load_module(&mut self, path: &[StaticIdentifier]) -> Result<(Arc<Text>, Vec<ModuleDefinitionItem<'static>>), Group<SemanticError>> {
        let text = match self.get_text(path) {
            Some(text) => text,
            None => return Err(Group::One(SemanticError::unresolved_item(
                Default::default(),
                path.to_vec(),
            ))),
        };
        let tokens = match Scanner::scan(&text.text) {
            Ok(tokens) => tokens,
            Err(error) => return Err(Group::One(SemanticError::scanner_error(error))),
        };
        match parse(&tokens, module) {
            Ok(items) => Ok((text.clone(), items.into_static())),
            Err(error_group) => Err(Group::new(
                error_group.extract_into_vec().into_iter()
                    .map(|item| SemanticError::parser_error(item))
                    .collect()
            )),
        }
    }
}

pub trait TextSourceWithDebug: TextSource + Debug {}

impl<T: TextSource + Debug> TextSourceWithDebug for T {}

use std::collections::HashMap;
use std::hash::BuildHasher;

impl<S: BuildHasher> TextSource for HashMap<Vec<StaticIdentifier>, String, S> {
    fn get_text(&mut self, path: &[StaticIdentifier]) -> Option<String> {
        self.get(path).map(String::clone)
    }
}
