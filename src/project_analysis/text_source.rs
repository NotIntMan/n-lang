use parser_basics::StaticIdentifier;
use syntax_parser::others::StaticPath;

pub trait TextSource {
    fn get_text(&mut self, path: &[StaticIdentifier]) -> Option<String>;
}

use std::collections::HashMap;
use std::hash::BuildHasher;

impl<S: BuildHasher> TextSource for HashMap<StaticPath, String, S> {
    fn get_text(&mut self, path: &[StaticIdentifier]) -> Option<String> {
        self.get(path).map(String::clone)
    }
}
