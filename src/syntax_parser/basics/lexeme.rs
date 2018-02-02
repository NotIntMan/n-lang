//! Примитивы распознавания лексем

use lexeme_scanner::{
    Token,
    TokenKind,
};

use super::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
    ParserErrorKind,
};

/**
    Примитив "Лексема".
    Сравнивает тип первой полученной лексемы с ожидаемым.

    Ничего не возвращает в случае успеха.
    В случае не совпадения типов возвращет ошибку типа `ExpectedGotKind`.
    В случае неудачи получения лексемы возвращает ошибку типа `UnexpectedEnd`.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct Lexeme(pub TokenKind);

impl<'a, 'b> LexemeParser<'a, 'b> for Lexeme {
    type Result = ();
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let kind = {
            let t = cursor.get_or(0, ParserErrorKind::UnexpectedEnd)?;
            t.kind.clone()
        };
        if kind == self.0 {
            cursor.next();
            Ok(())
        } else {
            cursor.parse_error_on(0, ParserErrorKind::ExpectedGotKind(self.0.clone(), kind))
        }
    }
}

/**
    Примитив "Такая же лексема" или "Именно такая лексема".
    Отличается от `Lexeme` тем, что проверяет не только тип, но и текст полученной лексемы.

    Как и `Lexeme`, ничего не возвращает в случае успеха.
    В случае не совпадения типов возвращет ошибку типа `ExpectedGotKindText`.
    В случае неудачи получения лексемы возвращает ошибку типа `UnexpectedEnd`.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct LexemeExact<'a>(pub TokenKind, pub &'a str);

impl<'a> LexemeExact<'a> {
    /// Создаёт новое правило разбора группы символов
    #[inline]
    pub const fn group(text: &'a str) -> Self {
        LexemeExact(TokenKind::SymbolGroup, text)
    }
}

impl<'a, 'b, 'c> LexemeParser<'a, 'b> for LexemeExact<'c> {
    type Result = ();
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let got = {
            let t = cursor.get_or(0, ParserErrorKind::UnexpectedEnd)?;
            (t.kind.clone(), t.text.clone())
        };
        if (self.0 == got.0) && (self.1 == got.1) {
            cursor.next();
            Ok(())
        } else {
            cursor.parse_error_on(0, ParserErrorKind::ExpectedGotKindText((self.0.clone(), self.1.to_string()), (got.0, got.1.to_string())))
        }
    }
}

/**
    Примитив "Извлечение лексемы".
    Отличается от `Lexeme` тем, что клонирует и возвращает лексему (`Token`) в случае успеха.

    В случае не совпадения типов возвращет ошибку типа `ExpectedGotKind`.
    В случае неудачи получения лексемы возвращает ошибку типа `UnexpectedEnd`.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct LexemeExtract(pub TokenKind);

impl<'a, 'b> LexemeParser<'a, 'b> for LexemeExtract {
    type Result = Token<'b>;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let r = {
            let t = cursor.get_or(0, ParserErrorKind::UnexpectedEnd)?;
            if t.kind == self.0 {
                Ok(t.clone())
            } else {
                Err(t.kind.clone())
            }
        };
        match r {
            Ok(t) => {
                cursor.next();
                Ok(t)
            }
            Err(kind) => cursor.parse_error_on(0, ParserErrorKind::ExpectedGotKind(self.0.clone(), kind)),
        }
    }
}
