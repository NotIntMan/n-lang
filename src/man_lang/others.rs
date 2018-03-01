use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    list,
    ParserResult,
    symbols,
};

pub fn path<'token, 'source>(input: &'token [Token<'source>], delimiter: &str) -> ParserResult<'token, 'source, Vec<&'source str>> {
    do_parse!(input,
        first: identifier >>
        others: opt!(do_parse!(
            apply!(symbols, delimiter) >>
            list: apply!(list, identifier, prepare!(symbols(delimiter))) >>
            (list)
        )) >>
        (match others {
            Some(mut vec) => {
                vec.insert(0, first);
                vec
            },
            None => vec![first],
        })
    )
}

pub fn property_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Vec<&'source str>> {
    path(input, ".")
}

pub fn module_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Vec<&'source str>> {
    path(input, "::")
}
