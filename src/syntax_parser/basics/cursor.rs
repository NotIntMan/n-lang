use std::ops::Index;
use std::ops::{
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Try,
};

use nom::{
    IResult,
    ErrorKind,
};

use lexeme_scanner::{
    SymbolPosition,
    Token,
};

use super::{
    ParserError,
    ParserErrorKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor<'a, T: 'a> {
    source: &'a [T],
    offset: usize,
}

impl<'a, T> Cursor<'a, T> {
    #[inline]
    pub fn new(source: &'a [T]) -> Self {
        Self { source, offset: 0 }
    }
    #[inline]
    pub fn offset(&self) -> usize { self.offset }
    #[inline]
    pub fn source(&self) -> &'a [T] { self.source }
    #[inline]
    pub fn ok<O, E>(self, value: O) -> IResult<Self, O, E> {
        IResult::Done(self, value)
    }
}

impl<'a, 'b> Cursor<'a, Token<'b>> {
    #[inline]
    pub fn position_at(&self, index: usize) -> SymbolPosition {
        let len = self.source.len();
        let i = self.offset + index;
        if i < len {
            self[i].pos.clone()
        } else {
            if len > 0 {
                self.source[len - 1].pos.clone()
            } else {
                SymbolPosition::default()
            }
        }
    }
    #[inline]
    pub fn err_at<O>(self, index: usize, kind: ParserErrorKind) -> IResult<Self, O, ParserError> {
        let err = ParserError::from_pos(kind, self.position_at(index));
        IResult::Error(ErrorKind::Custom(err))
    }
    #[inline]
    pub fn err<O>(self, kind: ParserErrorKind) -> IResult<Self, O, ParserError> {
        self.err_at(0, kind)
    }
    #[inline]
    pub fn get<O, S: ToString>(&self, index: usize, msg: S) -> CursorExtractResult<'a, Token<'b>, ParserError> {
        let i = self.offset + index;
        if i < self.source.len() {
            CursorExtractResult::Ok(self[index].clone())
        } else {
            CursorExtractResult::Err(self.clone().err_at(index, ParserErrorKind::unexpected_end_expected(msg)))
        }
    }
}

impl<'a, T> Index<usize> for Cursor<'a, T> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let i = self.offset + index;
        &self.source[i]
    }
}

impl<'a, T> AddAssign<usize> for Cursor<'a, T> {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.offset += rhs;
    }
}

impl<'a, T> Add<usize> for Cursor<'a, T> {
    type Output = Self;
    #[inline]
    fn add(mut self, rhs: usize) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a, T> SubAssign<usize> for Cursor<'a, T> {
    #[inline]
    fn sub_assign(&mut self, rhs: usize) {
        self.offset -= rhs;
    }
}

impl<'a, T> Sub<usize> for Cursor<'a, T> {
    type Output = Self;
    #[inline]
    fn sub(mut self, rhs: usize) -> Self::Output {
        self -= rhs;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursorExtractResult<'a, O: 'a, E> {
    Ok(O),
    Err(IResult<Cursor<'a, O>, O, E>),
}

impl<'a, O: 'a, E> Try for CursorExtractResult<'a, O, E> {
    type Ok = O;
    type Error = IResult<Cursor<'a, O>, O, E>;
    #[inline]
    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
            CursorExtractResult::Ok(v) => Ok(v),
            CursorExtractResult::Err(e) => Err(e),
        }
    }
    #[inline]
    fn from_ok(v: Self::Ok) -> Self { CursorExtractResult::Ok(v) }
    #[inline]
    fn from_error(v: Self::Error) -> Self { CursorExtractResult::Err(v) }
}
