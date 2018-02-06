#[macro_use]
extern crate nom;

use std::ops::Add;

use nom::{
    IResult,
    ErrorKind,
};

fn fold<F>(input: &[u8], f: F, init: u8) -> IResult<&[u8], u8>
    where F: Fn(u8, u8) -> u8 {
    let mut result = init;
    for i in input {
        result = f(result, *i);
    }
    IResult::Done(&[][..], result)
}

named!(sum ( &[u8] ) -> u8,
    apply!(fold, u8::add, 0)
);

#[test]
fn u8_folds_into_sum_correctly() {
    let input = [1, 2, 3, 5];
    assert_eq!(
        sum(&input),
        IResult::Done(&[][..], 11)
    );
}

named!(sum_after_8 ( &[u8] ) -> u8,
    do_parse!(
        take!(8) >>
        s: sum >>
        (s)
    )
);

#[test]
fn u8_sum_after_8_correctly() {
    let input = [1, 2, 3, 5, 6, 7, 8, 9, 10, 1];
    assert_eq!(
        sum_after_8(&input),
        IResult::Done(&[][..], 11)
    );
}

fn err(_: &[u8]) -> IResult<&[u8], ()> {
    IResult::Error(ErrorKind::Custom(42))
}

named!(err_after_8 ( &[u8] ) -> (),
    do_parse!(
        take!(8) >>
        err >>
        (())
    )
);

#[test]
fn u8_err_after_8_correctly() {
    let input = [1, 2, 3, 5, 6, 7, 8, 9, 10, 1];
    assert_eq!(
        err_after_8(&input),
        IResult::Error(ErrorKind::Custom(42))
    );
}
