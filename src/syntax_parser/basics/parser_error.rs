use std::fmt::{
    Display,
    Result as FResult,
    Formatter,
};

use std::cmp::{
    Ord,
    Ordering,
    PartialOrd,
};

use lexeme_scanner::{
    TokenKind,
    SymbolPosition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserErrorKind {
    UnexpectedEnd,
    ExpectedGotKind(TokenKind, TokenKind),
    ExpectedGotKindText((TokenKind, String), (TokenKind, String)),
    ExpectedGotMessage(String, TokenKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserErrorItem {
    pub kind: ParserErrorKind,
    pub pos: SymbolPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    One(ParserErrorItem),
    Many(Vec<ParserErrorItem>),
}

impl Display for ParserErrorKind {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            &ParserErrorKind::UnexpectedEnd => write!(f, "unexpected end"),
            &ParserErrorKind::ExpectedGotKind(ref exp, ref got) => write!(f, "expected: {:?}, got: {:?}", exp, got),
            &ParserErrorKind::ExpectedGotKindText(
                (ref exp_kind, ref exp_text), (ref got_kind, ref got_text)
            ) => write!(f, "expected: {:?}({:?}), got: {:?}({:?})", exp_kind, exp_text, got_kind, got_text),
            &ParserErrorKind::ExpectedGotMessage(ref exp, ref got) => write!(f, "expected: {}, got: {:?}", exp, got),
        }
    }
}

impl ParserErrorItem {
    fn from_pos(kind: ParserErrorKind, pos: SymbolPosition) -> Self {
        Self {
            kind,
            pos,
        }
    }
}

impl Display for ParserErrorItem {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "{} on {}", self.kind, self.pos)
    }
}

impl PartialOrd for ParserErrorItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.pos.partial_cmp(&other.pos)
    }
}

impl Ord for ParserErrorItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("Trying to sort error from different modules")
    }
}

impl ParserError {
    pub fn from_pos(kind: ParserErrorKind, pos: SymbolPosition) -> ParserError {
        ParserError::One(
            ParserErrorItem::from_pos(kind, pos)
        )
    }
    pub fn extract_into_vec(&self) -> Vec<ParserErrorItem> {
        match self {
            &ParserError::One(ref e) => vec![e.clone()],
            &ParserError::Many(ref v) => v.clone(),
        }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        let mut errors = self.extract_into_vec();
        errors.sort();
        writeln!(f, "There are some errors:")?;
        for (i, error) in errors.into_iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, error)?;
        }
        writeln!(f, "Solution of one of them may solve the problem.")
    }
}

