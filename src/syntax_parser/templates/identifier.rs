use lexeme_scanner::TokenKind;

use super::super::basics::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
    ParserErrorKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier;

impl<'a, 'b> LexemeParser<'a, 'b> for Identifier {
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
