use lexeme_scanner::{
    Token,
    TokenKind,
};

use syntax_parser::basics::parser::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
};

use super::parser_error::ParserErrorKind;

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

#[derive(Debug, Clone, PartialEq)]
pub struct LexemeExact<'a>(pub TokenKind, pub &'a str);

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

#[test]
fn lexeme_detects_correctly() {
    use helpers::iter_buffer::IterBuffer;
    use lexeme_scanner::{
        SymbolPosition,
        Token,
    };
    use env_logger::try_init;
    let _ = try_init();
    let mut buffer = IterBuffer::from_vec(vec![
        Token::new(
            TokenKind::Word,
            "first",
            SymbolPosition {
                offset: 0,
                line: 1,
                column: 1,
            },
        ),
        Token::new(
            TokenKind::EndOfInput,
            "",
            SymbolPosition {
                offset: 5,
                line: 1,
                column: 6,
            },
        ),
    ]);
    let mut cursor = buffer.cursor(0);
    Lexeme(TokenKind::Word).parse(&mut cursor).expect("a word");
    Lexeme(TokenKind::EndOfInput).parse(&mut cursor).expect("end of the input");
    assert_eq!(cursor.next(), None);
}

#[test]
fn lexeme_text_extracts_correctly() {
    use helpers::iter_buffer::IterBuffer;
    use lexeme_scanner::{
        SymbolPosition,
        Token,
    };
    use env_logger::try_init;
    let _ = try_init();
    let mut buffer = IterBuffer::from_vec(vec![
        Token::new(
            TokenKind::Word,
            "first",
            SymbolPosition {
                offset: 0,
                line: 1,
                column: 1,
            },
        ),
    ]);
    let mut cursor = buffer.cursor(0);
    const RULE: LexemeExtract = LexemeExtract(TokenKind::Word);
    let t = RULE.parse(&mut cursor).expect("a word");
    assert_eq!(t.text, "first");
    assert_eq!(cursor.next(), None);
}
