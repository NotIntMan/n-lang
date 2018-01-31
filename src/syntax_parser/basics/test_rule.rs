use lexeme_scanner::{
    TokenKind,
};

use super::lexeme::LexemeExact;

use super::parser::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
};

use super::parser_error::ParserErrorKind;

pub struct Number;

impl<'a, 'b> LexemeParser<'a, 'b> for Number {
    type Result = &'b str;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let r = {
            let t = cursor.get_or(0, ParserErrorKind::UnexpectedEnd)?;
            match t.kind {
                TokenKind::NumberLiteral { negative: _, fractional: _, radix: _ } => Ok(t.text),
                _ => Err(ParserErrorKind::ExpectedGotMessage("number".to_string(), t.kind.clone())),
            }
        };
        match r {
            Ok(t) => {
                cursor.next();
                Ok(t)
            },
            Err(k) => cursor.parse_error_on(0, k),
        }
    }
}

pub struct Sum;

const PLUS: LexemeExact = LexemeExact(TokenKind::SymbolGroup, "+");

impl<'a, 'b> LexemeParser<'a, 'b> for Sum {
    type Result = (&'b str, &'b str);
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let l = Number.parse(cursor)?;
        PLUS.parse(cursor)?;
        let r = Number.parse(cursor)?;
        Ok((l, r))
    }
}

#[test]
fn x() {}
