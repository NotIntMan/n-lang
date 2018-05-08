use parser_basics::{
    comma_list,
    Parser,
    ParserResult,
    symbols,
};
use language::others::{
    property_path,
    module_path,
};
use super::*;

pub fn property_access<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, ExpressionAST<'source>>,
) -> ParserResult<'token, 'source, ExpressionAST<'source>> {
    do_parse!(input,
        atomic: atom >>
        tail: opt!(do_parse!(
            apply!(symbols, ".") >>
            path: property_path >>
            (path)
        )) >>
        (match tail {
            Some(path) => ExpressionAST::PropertyAccess(Box::new(atomic), path),
            None => atomic,
        })
    )
}

parser_rule!(expression_set(i, atom: Parser<'token, 'source, ExpressionAST<'source>>) -> Vec<ExpressionAST<'source>> {
    do_parse!(i,
        apply!(symbols, "(") >>
        items: apply!(comma_list, atom) >>
        apply!(symbols, ")") >>
        (items)
    )
});

pub fn set<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, ExpressionAST<'source>>,
) -> ParserResult<'token, 'source, ExpressionAST<'source>> {
    expression_set(input, atom)
        .map(|mut items| if items.len() == 1 {
            items.swap_remove(0)
        } else {
            ExpressionAST::Set(items)
        })
}

pub fn function_call<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, ExpressionAST<'source>>,
) -> ParserResult<'token, 'source, ExpressionAST<'source>> {
    do_parse!(input,
        name: module_path >>
        args: apply!(expression_set, atom) >>
        (ExpressionAST::FunctionCall(name, args))
    )
}
