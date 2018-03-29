use std::fmt::Debug;
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

pub trait TextSource {
    fn get_text(&mut self, path: &[StaticIdentifier]) -> Option<String>;

    fn try_load_module(&mut self, path: &[StaticIdentifier]) -> Result<Vec<ModuleDefinitionItem<'static>>, Group<SemanticError>> {
        let text = match self.get_text(path) {
            Some(text) => text,
            None => return Err(Group::One(SemanticError::unresolved_item(
                Default::default(),
                path.to_vec(),
            ))),
        };
        let tokens = match Scanner::scan(&text) {
            Ok(tokens) => tokens,
            Err(error) => return Err(Group::One(SemanticError::scanner_error(error))),
        };
        match parse(&tokens, module) {
            Ok(items) => Ok(items.into_static()),
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
