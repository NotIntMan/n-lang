use std::ops::Range;

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
    RuleOption,
    RuleBranch,
    RuleRepeat,
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
            }
            Err(k) => cursor.parse_error_on(0, k),
        }
    }
}

type Plus<'a> = LexemeExact<'a>;

const PLUS: Plus = LexemeExact(TokenKind::SymbolGroup, "+");

pub struct Sum;

impl<'a, 'b> LexemeParser<'a, 'b> for Sum {
    type Result = (&'b str, &'static str, &'b str);
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let l = Number.parse(cursor)?;
        PLUS.parse(cursor)?;
        let r = Number.parse(cursor)?;
        Ok((l, "+", r))
    }
}

type OptionalSum = RuleOption<Sum>;

const OPTIONAL_SUM: OptionalSum = RuleOption(Sum);

type Minus<'a> = LexemeExact<'a>;

const MINUS: Minus = LexemeExact(TokenKind::SymbolGroup, "-");

pub struct Sub;

impl<'a, 'b> LexemeParser<'a, 'b> for Sub {
    type Result = (&'b str, &'static str, &'b str);
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let l = Number.parse(cursor)?;
        MINUS.parse(cursor)?;
        let r = Number.parse(cursor)?;
        Ok((l, "-", r))
    }
}

type SumOrSub = RuleBranch<Sum, Sub>;

const SUM_OR_SUB: SumOrSub = RuleBranch(Sum, Sub);

type Pluses<'a> = RuleRepeat<Plus<'a>, Range<usize>>;

const PLUSES: Pluses = RuleRepeat(PLUS, 2..6);

#[test]
fn sum_correctly_parses_input() {
    let mut buf = Scanner::scan("7 + 9")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    assert_eq!(
        Sum.parse(&mut cursor)
            .expect("Parsing result with no error"),
        ("7", "+", "9")
    );
    assert_eq!(
        cursor.next()
            .expect("Some(Token)")
            .kind,
        TokenKind::EndOfInput
    );
    assert_eq!(cursor.next(), None);
}

#[test]
fn optional_sum_correctly_parses_sum_input() {
    let mut buf = Scanner::scan("1 + 19")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    assert_eq!(
        OPTIONAL_SUM.parse(&mut cursor)
            .expect("Parsing result with no error"),
        Some(("1", "+", "19"))
    );
    assert_eq!(
        cursor.next()
            .expect("Some(Token)")
            .kind,
        TokenKind::EndOfInput
    );
    assert_eq!(cursor.next(), None);
}

#[test]
fn optional_sum_correctly_parses_sub_input() {
    let mut buf = Scanner::scan("1 - 19")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    assert_eq!(
        OPTIONAL_SUM.parse(&mut cursor)
            .expect("Parsing result with no error"),
        None
    );
    assert!(
        cursor.next()
            .expect("Some(Token)")
            .kind.is_number()
    );
}

#[test]
fn sum_or_sub_correctly_parses_sub_input() {
    let mut buf = Scanner::scan("10 + 1.9")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    assert_eq!(
        SUM_OR_SUB.parse(&mut cursor)
            .expect("Parsing result with no error"),
        ("10", "+", "1.9")
    );
    assert_eq!(
        cursor.next()
            .expect("Some(Token)")
            .kind,
        TokenKind::EndOfInput
    );
    assert_eq!(cursor.next(), None);
}

#[test]
fn sum_or_sub_correctly_parses_sum_input() {
    let mut buf = Scanner::scan("10 - 1.9")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    assert_eq!(
        SUM_OR_SUB.parse(&mut cursor)
            .expect("Parsing result with no error"),
        ("10", "-", "1.9")
    );
    assert_eq!(
        cursor.next()
            .expect("Some(Token)")
            .kind,
        TokenKind::EndOfInput
    );
    assert_eq!(cursor.next(), None);
}

#[test]
fn pluses_correctly_parses_input0() {
    let mut buf = Scanner::scan("+++")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    assert_eq!(
        PLUSES.parse(&mut cursor)
            .expect("Parsing result with no error")
            .len(),
        3
    );
    assert_eq!(
        cursor.next()
            .expect("Some(Token)")
            .kind,
        TokenKind::EndOfInput
    );
    assert_eq!(cursor.next(), None);
}

#[test]
fn pluses_correctly_parses_input1() {
    use env_logger::try_init;
    let _ = try_init();
    let mut buf = Scanner::scan("++++++")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    assert_eq!(
        PLUSES.parse(&mut cursor)
            .expect("Parsing result with no error")
            .len(),
        5
    );
    PLUS.parse(&mut cursor)
        .expect("Parsing result with no error");
    assert_eq!(
        cursor.next()
            .expect("Some(Token)")
            .kind,
        TokenKind::EndOfInput
    );
    assert_eq!(cursor.next(), None);
}

