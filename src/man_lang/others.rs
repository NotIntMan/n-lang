use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    list,
    ParserResult,
    symbols,
};

pub fn property_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Vec<&'source str>> {
    list(input, identifier, prepare!(symbols(".")))
}
