use language::{
    block,
    data_type,
    DataTypeAST,
};
use lexeme_scanner::Token;
use parser_basics::{
    comma_list,
    identifier,
    Identifier,
    item_position,
    keyword,
    symbol_position,
    symbols,
};
use parser_basics::ParserResult;
use super::*;

parser_rule!(type_of(i) -> DataTypeAST<'source> {
    do_parse!(i,
        apply!(symbols, ":") >>
        data_type: data_type >>
        (data_type)
    )
});

parser_rule!(result_type_of(i) -> DataTypeAST<'source> {
    do_parse!(i,
        apply!(symbols, ":") >>
        data_type: data_type >>
        is_array: opt!(apply!(symbols, "[]")) >>
        (if is_array.is_some() {
            data_type.array()
        } else {
            data_type
        })
    )
});

parser_rule!(argument(i) -> (Identifier<'source>, DataTypeAST<'source>) {
    do_parse!(i,
        name: identifier >>
        data_type: type_of >>
        ((name, data_type))
    )
});

parser_rule!(arguments(i) -> Vec<(Identifier<'source>, DataTypeAST<'source>)> {
    do_parse!(i,
        apply!(symbols, "(") >>
        argument_list: apply!(comma_list, argument) >>
        apply!(symbols, ")") >>
        (argument_list)
    )
});

pub fn function_definition<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, FunctionDefinitionAST<'source>> {
    alt!(input,
        do_parse!(
            begin: symbol_position >>
            apply!(keyword, "extern") >>
            apply!(keyword, "fn") >>
            name: identifier >>
            arguments: arguments >>
            result: opt!(result_type_of) >>
            pos: apply!(item_position, begin) >>
            (FunctionDefinitionAST {
                pos,
                name,
                arguments,
                result,
                body: FunctionBodyAST::External,
            })
        )
        | do_parse!(
            begin: symbol_position >>
            apply!(keyword, "fn") >>
            name: identifier >>
            arguments: arguments >>
            result: opt!(result_type_of) >>
            body: block >>
            pos: apply!(item_position, begin) >>
            (FunctionDefinitionAST {
                pos,
                name,
                arguments,
                result,
                body: FunctionBodyAST::Implementation(body),
            })
        )
    )
}
