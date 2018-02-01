/*!
    Основа парсера. Содержит примитивы распознавания, разбора и вспомогитальные типы и структуры.

    В основе структуры типов лежит типаж `LexemeParser`. Реализуя его, структура сообщает о том,
    что её можно использовать для синтаксического разбора лексем, сгенерированных модулем `lexeme_scanner`.

    Примитивы, расположенные в модулях `lexeme` и `rule`, реализуют типаж `LexemeParser`.
    Это значит, что их можно комбинировать с прочими правилами в любой последовательности и виде.
*/

pub mod lexeme;
pub mod parser;
pub mod parser_error;
pub mod rule;

pub use self::lexeme::{
    Lexeme,
    LexemeExact,
    LexemeExtract,
};

pub use self::parser::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
};

pub use self::parser_error::{
    ParserError,
    ParserErrorItem,
    ParserErrorKind,
};

pub use self::rule::{
    RuleBranch,
    RuleOption,
    RuleRepeat,
};

#[cfg(test)]
pub mod basics_tests;
