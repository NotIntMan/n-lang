use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    item_position,
    ParserResult,
    symbol_position,
};
use self::binary_operations::binary_expression;
use self::literals::literal;
use self::others::{
    function_call,
    property_access,
    set,
};
use self::unary_operations::unary_operation;
use super::*;

pub mod literals;
pub mod binary_operations;
pub mod unary_operations;
pub mod others;

parser_rule!(expression_atom(i) -> ExpressionAST<'source> {
    do_parse!(i,
        begin: symbol_position >>
        body: alt!(
            literal => { |x| ExpressionASTBody::Literal(x) } |
            apply!(function_call, expression) |
            apply!(set, expression) |
            identifier => { |x| ExpressionASTBody::Reference(x) }
        ) >>
        pos: apply!(item_position, begin) >>
        (ExpressionAST { body, pos })
    )
});

parser_rule!(unary_atom(i) -> ExpressionAST<'source> { property_access(i, expression_atom) });

parser_rule!(binary_atom(i) -> ExpressionAST<'source> { unary_operation(i, unary_atom) });

/// Функция, выполняющая разбор выражений
pub fn expression<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, ExpressionAST<'source>> {
    binary_expression(input, binary_atom)
}
