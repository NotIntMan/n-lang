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
