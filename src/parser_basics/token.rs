//! Набор примитивных правил распознавания токенов

use lexeme_scanner::{
    Token,
    TokenKindLess,
};

use super::{
    ParserInput,
    ParserResult,
    ParserErrorKind,
};

/**
    Правило "Какой-нибудь токен".
    Возвращает ссылку на первый найденный токен.

    В случае, если ввод пустой или содержит только токен типа `EndOfInput` возвращает ошибку типа `UnexpectedEnd`.
*/
pub fn some_token<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, &'a Token<'b>> {
    if input.is_empty() {
        return input.err(ParserErrorKind::unexpected_end());
    }
    input.ok_at(1, &input[0])
}

/**
    Правило "Токен".
    Сравнивает тип первого токена во вводе с данным.

    В случае совпадения возвращает ссылку на токен.
    В случае не совпадения возвращет ошибку типа `ExpectedGot`.
    В случае, если ввод пустой или содержит только токен типа `EndOfInput` возвращает ошибку типа `UnexpectedEnd`.
*/
pub fn token<'a, 'b>(input: &'a [Token<'b>], expected: TokenKindLess) -> ParserResult<'a, 'b, &'a Token<'b>> {
    if input.is_empty() {
        return input.err(ParserErrorKind::unexpected_end_expected(expected));
    }
    let got = input[0].kind.less();
    if expected == got {
        input.ok_at(1, &input[0])
    } else {
        input.err(ParserErrorKind::expected_got_kind(expected, got))
    }
}

/**
    Правило "Именно такой токен".
    Сравнивает тип и текст первого токена во вводе с данным.

    В случае совпадения возвращает ссылку на токен.
    В случае не совпадения возвращет ошибку типа `ExpectedGot`.
    В случае, если ввод пустой или содержит только токен типа `EndOfInput` возвращает ошибку типа `UnexpectedEnd`.
*/
pub fn exact_token<'a, 'b>(input: &'a [Token<'b>], expected_kind: TokenKindLess, expected_text: &str) -> ParserResult<'a, 'b, &'a Token<'b>> {
    if input.is_empty() {
        return input.err(ParserErrorKind::unexpected_end_expected(expected_kind));
    }
    let got_kind = input[0].kind.less();
    let got_text = input[0].text;
    if (expected_kind == got_kind) && (expected_text == got_text) {
        input.ok_at(1, &input[0])
    } else {
        input.err(ParserErrorKind::expected_got_kind_text(expected_kind, expected_text, got_kind, got_text))
    }
}

#[test]
fn x() {
    use nom;
    use parser_basics;
    use self::TokenKindLess::*;
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
    use lexeme_scanner::Scanner;
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
fn y() {
    use nom;
    use parser_basics;
    use self::TokenKindLess::*;
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
    use lexeme_scanner::Scanner;
    let buf = Scanner::scan("azaz +")
        .expect("Scanner result must be ok");
    let input = buf.as_slice();
    let err = xx(input)
        .to_result()
        .expect_err("Parser result must be err");
    let err = match err {
        nom::ErrorKind::Custom(e) => {
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
