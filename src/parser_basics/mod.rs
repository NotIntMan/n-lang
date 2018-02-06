/*!
    Основа парсера. Содержит примитивы распознавания, разбора и вспомогитальные типы и структуры.

    Примитивы, расположенные в модулях `token` и `rule`, представляют собой примитивные парсеры.
    Это значит, что их можно комбинировать с прочими правилами в любой последовательности и виде.

    В модулях `basic_rules` и `templates` располагаются примитивы грамматики,
    которые должны помочь в её построении,
    такие как "идентификатор", "ключевое слово", "список" и "обёртка".
*/

use nom::IResult;

use lexeme_scanner::Token;

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
    special_number_literal,
    string_literal,
    symbols,
};

pub use self::input::ParserInput;

pub type Parser<'a, 'b, O> = Fn(&'a [Token<'b>]) -> ParserResult<'a, 'b, O>;
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
