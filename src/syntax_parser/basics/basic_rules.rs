//! Набор примитивных правил для образования языка

use lexeme_scanner::TokenKind;

use super::super::basics::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
    ParserErrorKind,
};

/**
    Правило "Идентификатор".
    Ищет лексему типа `Word` и возвращает её текст в случае успеха.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicIdentifier;

impl<'a, 'b> LexemeParser<'a, 'b> for BasicIdentifier {
    type Result = &'b str;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let extract_result = {
            let t = cursor.get_or(0, ParserErrorKind::UnexpectedEnd)?;
            if t.kind == TokenKind::Word {
                Ok(t.text)
            } else {
                Err(t.kind.clone())
            }
        };
        match extract_result {
            Ok(text) => {
                cursor.next();
                Ok(text)
            },
            Err(kind) => {
                cursor.parse_error_on(0, ParserErrorKind::ExpectedGotKind(TokenKind::Word, kind))
            },
        }
    }
}

/**
    Правило "Ключевое слово".
    Ищет лексему типа `Word` с эквивалентным хранимому текстом.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicKeyword<'a>(&'a str);

impl<'a, 'b, 'c> LexemeParser<'a, 'b> for BasicKeyword<'c> {
    type Result = ();
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let extract_result = {
            let t = cursor.get_or(0, ParserErrorKind::UnexpectedEnd)?;
            if (t.kind == TokenKind::Word) && (t.text == self.0) {
                Ok(())
            } else {
                Err((t.kind.clone(), t.text.to_string()))
            }
        };
        match extract_result {
            Ok(text) => {
                cursor.next();
                Ok(text)
            },
            Err(kind_text) => {
                cursor.parse_error_on(0, ParserErrorKind::ExpectedGotKindText((TokenKind::Word, self.0.to_string()), kind_text))
            },
        }

    }
}
