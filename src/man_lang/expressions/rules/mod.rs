pub mod literals;
pub mod binary_operations;
pub mod unary_operators;

use lexeme_scanner::Token;
use parser_basics::{
    identifier_raw,
    ParserResult,
};
use super::*;
use self::literals::literal;

/// Функция, выполняющая разбор выражений
pub fn expression<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Expression<'source>> {
    alt!(input,
        literal => { |x| Expression::Literal(x) } |
        identifier_raw => { |x: &Token<'source>| Expression::Identifier(*x) }
    )
}
