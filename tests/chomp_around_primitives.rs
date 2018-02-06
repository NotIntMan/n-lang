#[macro_use]
extern crate chomp;

use chomp::prelude::{
    Input,
    ParseResult,
    Error,
    parse_only,
};

fn sum(input: &[u8]) -> ParseResult<&[u8], u8, Error<u8>> {
    let mut sum = 0;
    for i in input {
        sum += *i;
    }
    input.ret(sum)
}

#[test]
fn u8_sums_correctly() {
    assert_eq!(
        parse_only(sum, &[1, 2, 3][..]),
        Ok(6)
    );
}
