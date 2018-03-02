use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    ParserResult,
    symbols,
};

pub fn path<'token, 'source>(input: &'token [Token<'source>], delimiter: &str) -> ParserResult<'token, 'source, Vec<&'source str>> {
    do_parse!(input,
        first: identifier >>
        others: many0!(do_parse!(
            apply!(symbols, delimiter) >>
            element: identifier >>
            (element)
        )) >>
        ({
            let mut result = others;
            result.insert(0, first);
            #[cfg(feature="parser_trace")]
            trace!("Path found: {:?}", result);
            result
        })
    )
}

pub fn property_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Vec<&'source str>> {
    path(input, ".")
}

pub fn module_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Vec<&'source str>> {
    path(input, "::")
}
