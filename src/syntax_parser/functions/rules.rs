use indexmap::IndexMap;
use parser_basics::{
    comma_list,
    identifier,
    keyword,
    symbols,
};
use lexeme_scanner::Token;
use parser_basics::ParserResult;
use syntax_parser::compound_types::{
    data_type,
    DataType,
};
use syntax_parser::statements::block;
use super::*;

parser_rule!(type_of(i) -> DataType<'source> {
    do_parse!(i,
        apply!(symbols, ":") >>
        data_type: data_type >>
        (data_type)
    )
});

parser_rule!(argument(i) -> (&'source str, DataType<'source>) {
    do_parse!(i,
        name: identifier >>
        data_type: type_of >>
        ((name, data_type))
    )
});

parser_rule!(arguments(i) -> IndexMap<&'source str, DataType<'source>> {
    do_parse!(i,
        apply!(symbols, "(") >>
        argument_list: apply!(comma_list, argument) >>
        apply!(symbols, ")") >>
        ({
            let mut result = IndexMap::new();
            for (name, data_type) in argument_list {
                result.insert(name, data_type);
            }
            result
        })
    )
});

pub fn function_definition<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, FunctionDefinition<'source>> {
    alt!(input,
        do_parse!(
            apply!(keyword, "extern") >>
            apply!(keyword, "fn") >>
            name: identifier >>
            arguments: arguments >>
            result: opt!(type_of) >>
            (FunctionDefinition {
                name,
                arguments,
                result,
                body: FunctionBody::External,
            })
        )
        | do_parse!(
            apply!(keyword, "fn") >>
            name: identifier >>
            arguments: arguments >>
            result: opt!(type_of) >>
            body: block >>
            (FunctionDefinition {
                name,
                arguments,
                result,
                body: FunctionBody::Implementation(body),
            })
        )
    )
}