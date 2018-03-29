use std::fmt::Debug;
use std::sync::Arc;
use std::fmt;
use helpers::group::Group;
use helpers::into_static::IntoStatic;
use helpers::write_pad::display;
use lexeme_scanner::Scanner;
use parser_basics::{
    parse,
    StaticIdentifier,
};
use syntax_parser::modules::{
    module,
    ModuleDefinitionItem,
};
use syntax_parser::others::write_path;
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

#[derive(Clone)]
pub struct HashMapSource {
    map: HashMap<Vec<StaticIdentifier>, Arc<Text>>,
}

impl HashMapSource {
    pub fn new() -> Self {
        HashMapSource {
            map: HashMap::new(),
        }
    }
    pub fn simple_insert(&mut self, path: Vec<&str>, name: &str, text: &str) {
        self.map.insert(
            path.into_iter()
                .map(|name|
                    StaticIdentifier::new(name).into_static()
                )
                .collect(),
            Arc::new(Text {
                name: name.to_string(),
                text: text.to_string(),
            }),
        );
    }
}

impl fmt::Debug for HashMapSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut map = f.debug_map();
        for (path, text) in self.map.iter() {
            map.entry(
                &display(|s|
                    write_path(s, path.as_slice(), "::")
                ),
                &*text,
            );
        }
        map.finish()
    }
}

impl TextSource for HashMapSource {
    fn get_text(&mut self, path: &[StaticIdentifier]) -> Option<Arc<Text>> {
        self.map.get(path).map(Clone::clone)
    }
}
