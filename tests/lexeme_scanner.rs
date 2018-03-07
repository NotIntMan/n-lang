extern crate n_lang;
extern crate env_logger;

use n_lang::lexeme_scanner::{
    Scanner,
    ScannerError,
    ScannerErrorKind,
    Token,
    TokenKind,
    SymbolPosition,
};

#[allow(unused_imports)]
use env_logger::try_init;

#[test]
fn scans_alone_string_correctly() {
    let _ = try_init();
    let text = "\"my_text\"";
    let mut scanner = Scanner::new(text);
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::StringLiteral { length: 7 },
            text,
            SymbolPosition {
                offset: 0,
                line: 1,
                column: 1,
            },
        )
    );
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::EndOfInput,
            "",
            SymbolPosition {
                offset: 9,
                line: 1,
                column: 10,
            },
        )
    );
    assert_eq!(scanner.next(), None);
}

#[test]
fn scans_alone_binary_number_correctly() {
    let _ = try_init();
    let text = "0b1101";
    let mut scanner = Scanner::new(text);
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::NumberLiteral { negative: false, radix: 2, fractional: false },
            text,
            SymbolPosition::default(),
        )
    );
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::EndOfInput,
            "",
            SymbolPosition {
                offset: 6,
                line: 1,
                column: 7,
            },
        )
    );
    assert_eq!(scanner.next(), None);
}

#[test]
fn scans_alone_octal_number_correctly() {
    let _ = try_init();
    let text = "0112";
    let mut scanner = Scanner::new(text);
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::NumberLiteral { negative: false, radix: 8, fractional: false },
            text,
            SymbolPosition::default(),
        )
    );
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::EndOfInput,
            "",
            SymbolPosition {
                offset: 4,
                line: 1,
                column: 5,
            },
        )
    );
    assert_eq!(scanner.next(), None);
}

#[test]
fn scans_alone_decimal_number_correctly() {
    let _ = try_init();
    let text = "1";
    let mut scanner = Scanner::new(text);
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::NumberLiteral { negative: false, radix: 10, fractional: false },
            text,
            SymbolPosition::default(),
        )
    );
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::EndOfInput,
            "",
            SymbolPosition {
                offset: 1,
                line: 1,
                column: 2,
            },
        )
    );
    assert_eq!(scanner.next(), None);
}

#[test]
fn scans_alone_hexadecimal_number_correctly() {
    let _ = try_init();
    let text = "0x1F2c";
    let mut scanner = Scanner::new(text);
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::NumberLiteral { negative: false, radix: 16, fractional: false },
            text,
            SymbolPosition::default(),
        )
    );
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::EndOfInput,
            "",
            SymbolPosition {
                offset: 6,
                line: 1,
                column: 7,
            },
        )
    );
    assert_eq!(scanner.next(), None);
}

#[test]
fn scans_alone_negative_number_correctly() {
    let _ = try_init();
    let text = "-12";
    let mut scanner = Scanner::new(text);
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::NumberLiteral { negative: true, radix: 10, fractional: false },
            text,
            SymbolPosition::default(),
        )
    );
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::EndOfInput,
            "",
            SymbolPosition {
                offset: 3,
                line: 1,
                column: 4,
            },
        )
    );
    assert_eq!(scanner.next(), None);
}

#[test]
fn scans_alone_fractional_number_correctly() {
    let _ = try_init();
    let text = "0.3";
    let mut scanner = Scanner::new(text);
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::NumberLiteral { negative: false, radix: 10, fractional: true },
            text,
            SymbolPosition::default(),
        )
    );
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::EndOfInput,
            "",
            SymbolPosition {
                offset: 3,
                line: 1,
                column: 4,
            },
        )
    );
    assert_eq!(scanner.next(), None);
}

#[test]
fn scans_alone_wrong_number_correctly() {
    let _ = try_init();
    let text = "01F2c";
    let mut scanner = Scanner::new(text);
    assert_eq!(
        scanner.next(),
        Some(Err(ScannerError {
            kind: ScannerErrorKind::NotInRadix('F', 8),
            pos: SymbolPosition {
                offset: 2,
                line: 1,
                column: 3,
            },
        }))
    );
    assert_eq!(scanner.next(), None);
}

#[test]
fn scans_complex_text_correctly() {
    let _ = try_init();
    let text = "{ [] 1, 2, \"Azaz-kanzas\" +-*/ << !false NULL ThisIsMyFavoriteTable\n\n\r \r\t }";
    let mut scanner = Scanner::new(text);
    let mut pos = SymbolPosition::default();
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "{", pos.clone())
    );
    pos.step_str("{ ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "[", pos.clone())
    );
    pos.step_str("[");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "]", pos.clone())
    );
    pos.step_str("] ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::NumberLiteral { negative: false, radix: 10, fractional: false }, "1", pos.clone())
    );
    pos.step_str("1");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, ",", pos.clone())
    );
    pos.step_str(", ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::NumberLiteral { negative: false, radix: 10, fractional: false }, "2", pos.clone())
    );
    pos.step_str("2");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, ",", pos.clone())
    );
    pos.step_str(", ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::StringLiteral { length: 11 }, "\"Azaz-kanzas\"", pos.clone())
    );
    pos.step_str("\"Azaz-kanzas\" ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "+", pos.clone())
    );
    pos.step_str("+");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "-", pos.clone())
    );
    pos.step_str("-");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "*", pos.clone())
    );
    pos.step_str("*");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "/", pos.clone())
    );
    pos.step_str("/ ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "<<", pos.clone())
    );
    pos.step_str("<< ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "!", pos.clone())
    );
    pos.step_str("!");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::Word, "false", pos.clone())
    );
    pos.step_str("false ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::Word, "NULL", pos.clone())
    );
    pos.step_str("NULL ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::Word, "ThisIsMyFavoriteTable", pos.clone())
    );
    pos.step_str("ThisIsMyFavoriteTable\n\n\r \r\t ");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::SymbolGroup, "}", pos.clone())
    );
    pos.step_str("}");
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(TokenKind::EndOfInput, "", pos)
    );
    assert_eq!(scanner.next(), None);
}

#[test]
fn throw_and_format_error_correctly() {
    let _ = try_init();
    let text = "\"Azaz-";
    let mut scanner = Scanner::new(text);
    let err = scanner.next()
        .expect("Expected Some(Err(_))")
        .expect_err("Expected Err(_)")
    ;
    assert_eq!(
        err,
        ScannerError {
            kind: ScannerErrorKind::unexpected_end_expected("string body's symbol"),
            pos: SymbolPosition {
                offset: 6,
                line: 1,
                column: 7,
            },
        }
    );
    assert_eq!(
        err.to_string(),
        "Error: unexpected end of input, expected: string body's symbol on line 1, column 7"
    );
    assert_eq!(
        scanner.next(),
        None
    );
}

#[test]
fn coverts_into_token_iterator_correctly() {
    let _ = try_init();
    let text = "!2";
    let mut scanner = Scanner::new(text);
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::SymbolGroup,
            "!",
            SymbolPosition {
                offset: 0,
                line: 1,
                column: 1,
            },
        )
    );
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::NumberLiteral {
                negative: false,
                radix: 10,
                fractional: false,
            },
            "2",
            SymbolPosition {
                offset: 1,
                line: 1,
                column: 2,
            },
        )
    );
    assert_eq!(
        scanner.next(),
        Token::new_wrapped(
            TokenKind::EndOfInput,
            "",
            SymbolPosition {
                offset: 2,
                line: 1,
                column: 3,
            },
        )
    );
    assert_eq!(scanner.next(), None);
}
