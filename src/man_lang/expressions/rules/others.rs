use parser_basics::{
    comma_list,
    identifier,
    Parser,
    ParserResult,
    symbols,
};
use man_lang::others::property_path;
use super::*;

pub fn property_access<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, Expression<'source>>,
) -> ParserResult<'token, 'source, Expression<'source>> {
    do_parse!(input,
        atomic: atom >>
        tail: opt!(do_parse!(
            apply!(symbols, ".") >>
            path: property_path >>
            (path)
        )) >>
        (match tail {
            Some(path) => Expression::PropertyAccess(Box::new(atomic), path),
            None => atomic,
        })
    )
}

parser_rule!(expression_set(i, atom: Parser<'token, 'source, Expression<'source>>) -> Vec<Expression<'source>> {
    do_parse!(i,
        apply!(symbols, "(") >>
        items: apply!(comma_list, atom) >>
        apply!(symbols, ")") >>
        (items)
    )
});

pub fn set<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, Expression<'source>>,
) -> ParserResult<'token, 'source, Expression<'source>> {
    expression_set(input, atom)
        .map(|mut items| if items.len() == 1 {
            items.swap_remove(0)
        } else {
            Expression::Set(items)
        })
}

pub fn function_call<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, Expression<'source>>,
) -> ParserResult<'token, 'source, Expression<'source>> {
    do_parse!(input,
        name: identifier >>
        args: apply!(expression_set, atom) >>
        (Expression::FunctionCall(name, args))
    )
}
