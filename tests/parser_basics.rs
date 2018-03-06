#[macro_use]
extern crate n_lang;

#[macro_use]
extern crate nom;

use nom::ErrorKind;
use n_lang::lexeme_scanner::{
    TokenKindLess,
    Scanner,
};
use n_lang::parser_basics::{
    ParserErrorKind,
    token,
};

use self::TokenKindLess::*;

#[test]
fn simple_rule_makes_correctly() {
    parser_rule!(extract_text(i, kind: TokenKindLess) -> &'source str {
        do_parse!(i,
            x: apply!(token, kind) >>
            (x.text)
        )
    });
    parser_rule!(xx(input) -> (&'source str, &'source str) {
        do_parse!(input,
            w: apply!(extract_text, Word) >>
            n: apply!(extract_text, NumberLiteral) >>
            ((w, n))
        )
    });
    let buf = Scanner::scan("azaz 3")
        .expect("Scanner result must be ok");
    let input = buf.as_slice();
    assert_eq!(
        xx(input)
            .to_result()
            .expect("Parser result must be ok"),
        ("azaz", "3")
    );
}

#[test]
fn simple_rule_makes_correctly_2() {
    parser_rule!(extract_text(i, kind: TokenKindLess) -> &'source str {
        do_parse!(i,
            x: apply!(token, kind) >>
            (x.text)
        )
    });
    parser_rule!(xx(input) -> (&'source str, &'source str) {
        do_parse!(input,
            w: apply!(extract_text, Word) >>
            n: apply!(extract_text, NumberLiteral) >>
            ((w, n))
        )
    });
    let buf = Scanner::scan("azaz +")
        .expect("Scanner result must be ok");
    let input = buf.as_slice();
    let err = xx(input)
        .to_result()
        .expect_err("Parser result must be err");
    let err = match err {
        ErrorKind::Custom(e) => {
            let vec = e.extract_into_vec();
            assert_eq!(vec.len(), 1);
            vec[0].clone()
        }
        e => panic!("Got wrong error: {:?}", e),
    };
    assert_eq!(
        err.kind,
        ParserErrorKind::expected_got_kind(NumberLiteral, SymbolGroup)
    );
    assert_eq!(
        err.pos
            .expect("Error's position must be Some(_)")
            .offset,
        5
    );
}
