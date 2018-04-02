use helpers::group::Group;
use lexeme_scanner::Scanner;
use parser_basics::parse;
use syntax_parser::modules::module;
use super::error::SemanticError;
use super::project::ModulePathSlice;
use super::source::TextSource;
use super::module::Module;

fn try_load_module<S: TextSource>(source: &S, path: &ModulePathSlice) -> Result<Module, Group<SemanticError>> {
    let text = match source.get_text(path) {
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
        Ok(items) => Ok(Module::from_def(text.clone(), items)),
        Err(error_group) => Err(Group::new(
            error_group.extract_into_vec().into_iter()
                .map(|item| SemanticError::parser_error(item))
                .collect()
        )),
    }
}

pub fn resolve<S>(source: S) -> Group<SemanticError>
    where S: TextSource {
    let root = match try_load_module(&source, &[][..]) {
        Ok(x) => x,
        Err(group) => return group,
    };
    unimplemented!()
}
