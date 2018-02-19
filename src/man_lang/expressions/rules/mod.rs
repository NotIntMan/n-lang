pub mod literals;
pub mod binary_operations;
pub mod unary_operations;
pub mod others;

use lexeme_scanner::Token;
use parser_basics::{
    identifier_raw,
    ParserResult,
};
use super::*;
use self::literals::literal;
use self::binary_operations::binary_expression;
use self::unary_operations::unary_operation;
use self::others::{
    property_access,
    set,
    function_call,
};

parser_rule!(expression_atom(i) -> Expression<'source> {
    alt!(i,
        literal => { |x| Expression::Literal(x) } |
        apply!(function_call, expression) |
        apply!(set, expression) |
        identifier_raw => { |x: &Token<'source>| Expression::Identifier(*x) }
    )
});

parser_rule!(unary_atom(i) -> Expression<'source> { property_access(i, expression_atom) });

parser_rule!(binary_atom(i) -> Expression<'source> { unary_operation(i, unary_atom) });

/// Функция, выполняющая разбор выражений
pub fn expression<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Expression<'source>> {
    binary_expression(input, binary_atom)
}
