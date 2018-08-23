use lexeme_scanner::Token;
use nom::{
    ErrorKind,
    IResult,
};
use super::{
    new_error,
    new_error_without_pos,
    ParserError,
    ParserErrorKind,
};

/**
    Типаж, который позволит круто и просто реализовать генерацию значений для возврата из парсера.

    Для этого у типала есть два метода: `ok` и `err`, а так же два ассоциированных типа.

    В качестве примера приведем реализацию этого типажа для синонима `&[u8]`.

    ```rust,no_run

    # extern crate nom;
    # use nom::{IResult, ErrorKind};
    # extern crate n_lang;
    # use n_lang::lexeme_scanner::Token;
    # use n_lang::parser_basics::{new_error_without_pos, ParserInput, ParserError, ParserErrorKind};

    struct Input<'a>(&'a [u8]);

    # fn main() {

    impl<'a> ParserInput for Input<'a> {
        type Error = ParserError<'a>;
        type ErrorKind = ParserErrorKind<'a>;
        fn ok_at<T>(self, processed: usize, value: T) -> IResult<Self, T, Self::Error> {
            IResult::Done(Input(&self.0[processed..]), value)
        }
        fn err_at<T>(self, position: usize, kind: Self::ErrorKind) -> IResult<Self, T, Self::Error> {
            IResult::Error(ErrorKind::Custom(new_error_without_pos(kind)))
        }
    }

    # }
    ```

    Как видно в примере, в методах `ok` и `err` мы сгенерировали значения,
    отображающие успех и ошибку разбора соответственно.

    Теперь у нас есть возможность легко копировать и передавать наш ввод потому, что он имеет элементарный тип,
    и генерировать результат разбора на основе этого же ввода потому, что мы реализовали типаж.
    Мы восхитительны, не так ли?!
*/
pub trait ParserInput: Sized {
    type Error;
    type ErrorKind;

    fn ok_at<T>(self, processed: usize, value: T) -> IResult<Self, T, Self::Error>;
    fn err_at<T>(self, position: usize, kind: Self::ErrorKind) -> IResult<Self, T, Self::Error>;

    fn ok<T>(self, value: T) -> IResult<Self, T, Self::Error> { self.ok_at(0, value) }
    fn err<T>(self, kind: Self::ErrorKind) -> IResult<Self, T, Self::Error> { self.err_at(0, kind) }
}

impl<'token, 'source> ParserInput for &'token [Token<'source>] {
    type Error = ParserError<'source>;
    type ErrorKind = ParserErrorKind<'source>;
    fn ok_at<T>(self, processed: usize, value: T) -> IResult<Self, T, Self::Error> {
        IResult::Done(&self[processed..], value)
    }
    fn err_at<T>(self, position: usize, kind: Self::ErrorKind) -> IResult<Self, T, Self::Error> {
        let len = self.len();
        let error = if position < len {
            new_error(kind, self[position].pos)
        } else {
            if len > 0 {
                let t = &self[len - 1];
                let mut pos = t.pos;
                pos.step_str(t.text);
                new_error(kind, pos)
            } else {
                new_error_without_pos(kind)
            }
        };
        IResult::Error(ErrorKind::Custom(error))
    }
    fn ok<T>(self, value: T) -> IResult<Self, T, Self::Error> {
        IResult::Done(&self, value)
    }
}
