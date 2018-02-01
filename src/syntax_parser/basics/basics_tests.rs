use lexeme_scanner::{
    Scanner,
    TokenKind,
};

use super::{
    LexemeExact,
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
    ParserErrorKind,
};

pub struct Number;

impl<'a, 'b> LexemeParser<'a, 'b> for Number {
    type Result = &'b str;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let r = {
            let t = cursor.get_or(0, ParserErrorKind::UnexpectedEnd)?;
            if t.kind.is_number() {
                Ok(t.text)
            } else {
                Err(ParserErrorKind::ExpectedGotMessage("number".to_string(), t.kind.clone()))
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

const PLUS: LexemeExact = LexemeExact(TokenKind::SymbolGroup, "+");

pub struct Sum;

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
fn sum_correctly_parses_input() {
    let mut buf = Scanner::scan("7 + 9")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    assert_eq!(
        Sum.parse(&mut cursor)
            .expect("Parsing result with no error"),
        ("7", "9")
    );
    assert_eq!(
        cursor.next()
            .expect("Some(Token)")
            .kind,
        TokenKind::EndOfInput
    );
    assert_eq!(cursor.next(), None);
}
