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
pub mod parse_macro;
#[macro_use]
pub mod rule_macro;
pub mod token;
#[macro_use]
pub mod templates;

pub use self::basic_rules::{
    braced_expression_literal,
    end_of_input,
    identifier,
    Identifier,
    identifier_raw,
    item_position,
    keyword,
    not_keyword_identifier,
    number_literal,
    NumberLiteralSpec,
    none,
    special_number_literal,
    string_literal,
    symbols,
    u32_literal,
    symbol_position,
};

pub use self::input::ParserInput;

pub type Parser<'token, 'source, O> = fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, O>;
pub type ParserResult<'token, 'source, O> = IResult<&'token [Token<'source>], O, ParserError<'source>>;

pub use self::token::{
    exact_token,
    some_token,
    token,
};

pub use self::parser_error::{
    new_error,
    new_error_without_pos,
    ParserError,
    ParserErrorItem,
    ParserErrorKind,
    ParserErrorTokenInfo,
};

pub use self::templates::{
    comma_list,
    list,
    round_wrap,
    rounded_comma_list,
    symbol_wrap,
    wrap,
};

/// Запускает разбор переданного среза токенов и преобразует результат в стандартный `Result`
pub fn parse<'token, 'source, O>(input: &'token [Token<'source>], parser: Parser<'token, 'source, O>) -> Result<O, ParserError<'source>> {
    match parser(input) {
        IResult::Done(_, result) => Ok(result),
        IResult::Incomplete(_) => {
            let kind = ParserErrorKind::unexpected_end();
            Err(new_error_without_pos(kind))
        },
        IResult::Error(e) => match e {
            ErrorKind::Custom(e) => Err(e),
            other => {
                let msg = other.description();
                let kind = ParserErrorKind::custom_error(msg);
                Err(new_error_without_pos(kind))
            },
        },
    }
}
