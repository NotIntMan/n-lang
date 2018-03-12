//use std::ptr::Unique;
//use lexeme_scanner::{
//    Scanner,
//    ScannerError,
//};
//use parser_basics::{
//    parse,
//    ParserError,
//};
//use syntax_parser::modules::{
//    module,
//    ModuleDefinitionItem,
//};
//
//#[derive(Debug, Clone)]
//pub struct Module<'name, 'source> {
//    source: Vec<&'name str>,
//    items: Vec<ModuleDefinitionItem<'source>>,
//}
//
//impl Drop for Module {
//    fn drop(&mut self) {
//        // Уничтожение source
//        Box::from_unique(self.source)
//    }
//}
//
//#[derive(Debug, Clone, PartialEq, Eq)]
//pub enum ModuleParseError {
//    ScannerError(ScannerError),
//    ParserError(ParserError),
//}
//
//fn parse_module<S: ToString>(text: S) -> Result<ModuleDefinitionItem<'static>, ModuleParseError> {
//    let source = Box::into_unique(text.to_string().into_boxed_str());
//    let source_ref= unsafe {&*source.as_ptr()};
//    let tokens = match Scanner::scan(source_ref) {
//        Ok(x) => x,
//        Err(e) => return Err(ModuleParseError::ScannerError(e)),
//    };
//    let items = match parse(tokens.as_slice(), module) {
//        Ok(x) => x,
//        Err(e) => return Err(ModuleParseError::ParserError(e)),
//    };
//    Ok(Module {
//        source,
//        items,
//    })
//}
//
//pub trait ModuleResolver {
//    fn resolve(connector: &[&str], target: &[&str]) -> Result<ModuleDefinitionItem<'static>, ModuleParseError>;
//    fn read(path: &[&str]) -> Result<ModuleDefinitionItem<'static>, ModuleParseError>;
//}
//
//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct Project<Resolver> {
//    data_source: Resolver,
////    items:
//}
use std::path::PathBuf;
use indexmap::IndexMap;

pub mod module_path;
pub mod names_storage;

use self::module_path::ModulePath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    path: ModulePath,
    names: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    root: Module,
    // Это свойство планируется применять в реализации подключаемых библиотек (в т.ч. стандартной библиотеки СУБД)
    additional_modules: Vec<Module>,
    sources: IndexMap<PathBuf, String>,
}
