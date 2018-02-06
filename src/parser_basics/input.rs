use nom::{
    IResult,
    ErrorKind,
};

use lexeme_scanner::Token;

use super::{
    ParserError,
    ParserErrorKind,
};

pub trait ParserInput: Sized {
    type Error;
    type ErrorKind;
    fn ok<T>(self, processed: usize, value: T) -> IResult<Self, T, Self::Error>;
    fn err<T>(self, position: usize, kind: Self::ErrorKind) -> IResult<Self, T, Self::Error>;
}

impl<'a, 'b> ParserInput for &'a [Token<'b>] {
    type Error = ParserError;
    type ErrorKind = ParserErrorKind;
    fn ok<T>(self, processed: usize, value: T) -> IResult<Self, T, Self::Error> {
        IResult::Done(&self[processed..], value)
    }
    fn err<T>(self, position: usize, kind: Self::ErrorKind) -> IResult<Self, T, Self::Error> {
        let len = self.len();
        let error = if position < len {
            ParserError::new(kind, self[position].pos)
        } else {
            if len > 0 {
                let t = &self[len - 1];
                let mut pos = t.pos;
                pos.step_str(t.text);
                ParserError::new(kind, pos)
            } else {
                ParserError::new_without_pos(kind)
            }
        };
        IResult::Error(ErrorKind::Custom(error))
    }
}
