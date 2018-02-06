use nom::{
    IResult,
    ErrorKind,
};

use lexeme_scanner::Token;

use super::{
    ParserError,
    ParserErrorKind,
};

/**
    Типаж, который позволит круто и просто реализовать генерацию значений для возврата из парсера.

    Для этого у типала есть два метода: `ok` и `err`, а так же два ассоциированных типа.

    В качестве примера приведем реализацию этого типажа для `&[Token]`.

    ```rust,no_run
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
