/*!
    Основа парсера. Содержит примитивы распознавания, разбора и вспомогитальные типы и структуры.

    В основе структуры типов лежит типаж `LexemeParser`. Реализуя его, структура сообщает о том,
    что её можно использовать для синтаксического разбора лексем, сгенерированных модулем `lexeme_scanner`.

    Примитивы, расположенные в модулях `lexeme` и `rule`, реализуют типаж `LexemeParser`.
    Это значит, что их можно комбинировать с прочими правилами в любой последовательности и виде.

    В модуле `constants` располагаются константные правила, такие как последовательности специальных символов.

    В модулях `basic_rules` и `templates` располагаются примитивы грамматики,
    которые должны помочь в её построении,
    такие как "идентификатор", "ключевое слово", "список" и "обёртка".
*/

pub mod basic_rules;
pub mod constants;
pub mod lexeme;
pub mod parser;
pub mod parser_error;
#[macro_use]
pub mod rule;
pub mod templates;

pub use self::basic_rules::{
    BasicIdentifier,
    BasicKeyword,
};

pub use self::constants::{
    CLOSING_BRACES_BRACKET,
    CLOSING_ROUND_BRACKET,
    CLOSING_TRIANGULAR_BRACKET,
    OPENING_BRACES_BRACKET,
    OPENING_ROUND_BRACKET,
    OPENING_TRIANGULAR_BRACKET,
    COMMA,
    ConstSymbol,
    SEMICOLON,
};

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

pub use self::templates::{
    TemplateList,
    TemplateWrap
};

#[cfg(test)]
pub mod basics_tests;
