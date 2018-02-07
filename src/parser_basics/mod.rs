/*!
    Основа парсера. Содержит примитивы распознавания, разбора и вспомогитальные типы и структуры.

    Примитивы, расположенные в модулях `token` и `rule`, представляют собой примитивные парсеры.
    Это значит, что их можно комбинировать с прочими правилами в любой последовательности и виде.

    В модулях `basic_rules` и `templates` располагаются примитивы грамматики,
    которые должны помочь в её построении,
    такие как "идентификатор", "ключевое слово", "список" и "обёртка".
*/

use nom::{
    IResult,
    ErrorKind,
};

use lexeme_scanner::{
    Token,
};

pub mod basic_rules;
pub mod input;
pub mod parser_error;
#[macro_use]
pub mod rule_macro;
pub mod token;
#[macro_use]
pub mod templates;

pub use self::basic_rules::{
    braced_expression_literal,
    identifier,
    keyword,
    number_literal,
    NumberLiteralSpec,
    none,
    special_number_literal,
    string_literal,
    symbols,
    u32_literal,
};

pub use self::input::ParserInput;

pub type Parser<'a, 'b, O> = fn(&'a [Token<'b>]) -> ParserResult<'a, 'b, O>;
pub type ParserResult<'a, 'b, O> = IResult<&'a [Token<'b>], O, ParserError>;

pub use self::token::{
    exact_token,
    some_token,
    token,
};

pub use self::parser_error::{
    ParserError,
    ParserErrorItem,
    ParserErrorKind,
};

pub use self::templates::{
    list,
    round_wrap,
    symbol_wrap,
    wrap,
};

/// Запускает разбор переданного среза токенов и преобразует результат в стандартный `Result`
pub fn parse<'token, 'source, O>(input: &'token [Token<'source>], parser: Parser<'token, 'source, O>) -> Result<O, ParserError> {
    match parser(input) {
        IResult::Done(_, result) => Ok(result),
        IResult::Incomplete(_) => {
            let kind = ParserErrorKind::unexpected_end();
            Err(ParserError::new_without_pos(kind))
        },
        IResult::Error(e) => match e {
            ErrorKind::Custom(e) => Err(e),
            other => {
                let msg = other.description();
                let kind = ParserErrorKind::custom_error(msg);
                Err(ParserError::new_without_pos(kind))
            },
        },
    }
}
