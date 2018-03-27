use std::fmt::Debug;
use parser_basics::StaticIdentifier;

pub trait TextSource {
    fn get_text(&mut self, path: &[StaticIdentifier]) -> Option<String>;
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
