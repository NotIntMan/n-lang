use std::sync::Arc;
use helpers::group::Group;
use helpers::into_static::IntoStatic;
use lexeme_scanner::Scanner;
use parser_basics::parse;
use syntax_parser::modules::{
    module,
    ModuleDefinitionItem,
};
use super::error::SemanticError;
use super::project::ModulePathSlice;
use super::source::{
    Text,
    TextSource,
};

fn try_load_module<S: TextSource>(source: &S, path: &ModulePathSlice) -> Result<(Arc<Text>, Vec<ModuleDefinitionItem<'static>>), Group<SemanticError>> {
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
        Ok(items) => Ok((text.clone(), items.into_static())),
        Err(error_group) => Err(Group::new(
            error_group.extract_into_vec().into_iter()
                .map(|item| SemanticError::parser_error(item))
                .collect()
        )),
    }
}

pub fn resolve<S>(source: S) -> Group<SemanticError>
    where S: TextSource {
    let (text, defs) = match try_load_module(&source, &[][..]) {
        Ok(x) => x,
        Err(group) => return group,
    };
    unimplemented!()
}
