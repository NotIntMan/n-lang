//! Набор примитивных правил для образования языка

use std::ops::RangeFull;

use helpers::num_range::NumRange;

use lexeme_scanner::{
    Token,
    TokenKind,
};

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

/// Функция сравнения ключевых слов. Игнорирует регистр.
pub fn compare_words(a: &str, b: &str) -> bool {
    let left = a.chars().flat_map(|c| c.to_lowercase());
    let right = b.chars().flat_map(|c| c.to_lowercase());
    left.eq(right)
}

/**
    Правило "Ключевое слово".
    Ищет лексему типа `Word` с эквивалентным хранимому текстом.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicKeyword<'a>(pub &'a str);

impl<'a, 'b, 'c> LexemeParser<'a, 'b> for BasicKeyword<'c> {
    type Result = ();
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let extract_result = {
            let t = cursor.get_or(0, ParserErrorKind::UnexpectedEnd)?;
            if (t.kind == TokenKind::Word) && compare_words(t.text, self.0) {
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

/**
    Правило "Число".
    Ищет лексему типа `NumberLiteral` с эквивалентным хранимому текстом.
    К числу могут предъявляться определённые требования.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicNumber<T: NumRange<u32>> {
    negative: Option<bool>,
    fractional: Option<bool>,
    radix: T,
}

impl BasicNumber<RangeFull> {
    #[inline]
    pub const fn simple() -> Self {
        Self {
            negative: None,
            fractional: None,
            radix: ..,
        }
    }
    #[inline]
    pub const fn positive_integer() -> Self {
        Self {
            negative: Some(false),
            fractional: Some(false),
            radix: ..,
        }
    }
}

impl<T: NumRange<u32>> BasicNumber<T> {
    pub fn check(&self, negative: bool, fractional: bool, radix: u32) -> bool {
        if let Some(n) = self.negative {
            if negative != n { return false; }
        }
        if let Some(f) = self.fractional {
            if fractional != f { return false; }
        }
        self.radix.is_contains(&radix)
    }
}

impl<'a, 'b, T: NumRange<u32>> LexemeParser<'a, 'b> for BasicNumber<T> {
    type Result = Token<'b>;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
         let r = {
            let t = cursor.get_or(0, ParserErrorKind::UnexpectedEnd)?;
            match &t.kind {
                &TokenKind::NumberLiteral { negative, fractional, radix, } => {
                    if self.check(negative, fractional, radix) {
                        Ok(t.clone())
                    } else {
                        Err((t.kind.clone(), t.text.to_string()))
                    }
                },
                _ => Err((t.kind.clone(), t.text.to_string())),
            }
        };
        match r {
            Ok(token) => {
                cursor.next();
                Ok(token)
            },
            Err(got) => return cursor.parse_error_on(0, ParserErrorKind::ExpectedGotMessage("number".to_string(), got)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicUSizeLiteral;

impl<'a, 'b> LexemeParser<'a, 'b> for BasicUSizeLiteral {
    type Result = usize;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let t = BasicNumber::positive_integer().parse(cursor)?;
        // TODO Сделать нормальную ошибку вместо паники
        let r = t.text.parse::<usize>().expect("Usize must able to be parsed from positive integer literal");
        Ok(r)
    }
}
