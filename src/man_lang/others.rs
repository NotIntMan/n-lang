use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    list,
    ParserResult,
    symbols,
};

pub fn property_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Vec<&'source str>> {
    do_parse!(input,
        first: identifier >>
        others: opt!(do_parse!(
            apply!(symbols, ".") >>
            list: apply!(list, identifier, prepare!(symbols("."))) >>
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
