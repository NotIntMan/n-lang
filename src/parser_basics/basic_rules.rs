//! Набор примитивных правил для образования языка

use std::ops::Range;

use nom::IResult;

use lexeme_scanner::{
    Token,
    TokenKind,
    TokenKindLess,
};
use super::{
    ParserErrorKind,
    ParserInput,
    ParserResult,
    token,
    exact_token,
};

/**
    Правило "Ничего".
    Ничего не ожидает, ничего не возвращает.
    Ошибкам взяться неоткуда.
*/
pub fn none<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, ()> {
    input.ok(())
}

/**
    Правило "Идентификатор".
    Ищет токен типа `Word` и возвращает его текст в случае успеха.
*/
pub fn identifier<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, &'b str> {
    token(input, TokenKindLess::Word)
        .map(|token| token.text)
}

/**
    Правило "Сырой идентификатор".
    Ищет токен типа `Word` и возвращает ссылку на него в случае успеха.
*/
pub fn identifier_raw<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, &'a Token<'b>> {
    token(input, TokenKindLess::Word)
}

array!(pub const KEYWORD_LIST: &'static str =
    "as",
    "using",
    "on",
    "natural",
    "inner",
    "cross",
    "left",
    "right",
    "full",
    "where",
    "group",
    "having",
    "order",
    "limit",
);

/**
    Правило "Не ключевой идентификатор".
    Ищет токен типа `Word` и возвращает ссылку на него в случае успеха.
    Если текст токена содержится в списке ключевых слов `KEYWORD_LIST`, возвращает ошибку `ExpectedGot`.
*/
pub fn not_keyword_identifier<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, &'b str> {
    match identifier(input) {
        IResult::Done(new_input, result) => {
            if KEYWORD_LIST.contains(&result) {
                input.err(ParserErrorKind::expected_got_description("not keyword identifier", TokenKindLess::Word, result))
            } else {
                new_input.ok(result)
            }
        },
        IResult::Incomplete(n) => IResult::Incomplete(n),
        IResult::Error(e) => IResult::Error(e),
    }
}

/// Функция сравнения ключевых слов. Игнорирует регистр.
pub fn compare_words(a: &str, b: &str) -> bool {
    let left = a.chars().flat_map(|c| c.to_lowercase());
    let right = b.chars().flat_map(|c| c.to_lowercase());
    left.eq(right)
}

/**
    Правило "Ключевое слово".
    Ищет токен типа `Word` с текстом, эквивалентным данному.
    Возвращает ссылку на токен в случае успеха.
    Игнорирует регистр слова.
*/
pub fn keyword<'a, 'b>(input: &'a [Token<'b>], expected_text: &str) -> ParserResult<'a, 'b, &'a Token<'b>> {
    match token(input, TokenKindLess::Word) {
        IResult::Done(i, output) => {
            if !compare_words(output.text, expected_text) {
                return input.err(ParserErrorKind::expected_got_kind_text(TokenKindLess::Word, expected_text, TokenKindLess::Word, output.text))
            }
            i.ok(output)
        },
        IResult::Incomplete(n) => IResult::Incomplete(n),
        IResult::Error(e) => IResult::Error(e),
    }
}

/**
    Правило "Числовой литерал".
    Ищет токен типа `NumberLiteral` и возвращает его в случае успеха.
*/
pub fn number_literal<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, &'a Token<'b>> {
    token(input, TokenKindLess::NumberLiteral)
}

/**
    Правило "Строковый литерал".
    Ищет токен типа `StringLiteral` и возвращает его в случае успеха.
*/
pub fn string_literal<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, &'a Token<'b>> {
    token(input, TokenKindLess::StringLiteral)
}

/**
    Правило "Литерал выражения".
    Ищет токен типа `BracedExpressionLiteral` и возвращает его в случае успеха.
*/
pub fn braced_expression_literal<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, &'a Token<'b>> {
    token(input, TokenKindLess::BracedExpressionLiteral)
}

/**
    Правило "Специальные символы".
    Ищет токен типа `SymbolGroup` с текстом, эквивалентным данному.
    Ничего не возвращает в случае успеха.
*/
pub fn symbols<'a, 'b>(input: &'a [Token<'b>], text: &str) -> ParserResult<'a, 'b, ()> {
    exact_token(input, TokenKindLess::SymbolGroup, text)
        .map(|_| ())
}

/// Построитель (builder) спецификации для правила "Специальный числовой литерал".
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NumberLiteralSpec {
    pub negative: Option<bool>,
    pub fractional: Option<bool>,
    pub radix: Option<Range<u32>>,
}

impl NumberLiteralSpec {
    /// Создаёт новую спецификацию с заполненными полями значениями по умолчанию.
    #[inline]
    pub fn new() -> Self { NumberLiteralSpec::default() }
    /// Устанавливает значение "negative" в спецификации в `Some(value)`
    #[inline]
    pub fn set_negative(mut self, value: bool) -> Self { self.negative = Some(value); self }
    /// Устанавливает значение "negative" в спецификации в `None`
    #[inline]
    pub fn clear_negative(mut self) -> Self { self.negative = None; self }
    /// Устанавливает значение "fractional" в спецификации в `Some(value)`
    #[inline]
    pub fn set_fractional(mut self, value: bool) -> Self { self.fractional = Some(value); self }
    /// Устанавливает значение "fractional" в спецификации в `None`
    #[inline]
    pub fn clear_fractional(mut self) -> Self { self.fractional = None; self }
    /// Устанавливает значение "radix" в спецификации в `Some(value)`
    #[inline]
    pub fn set_radix(mut self, value: Range<u32>) -> Self { self.radix = Some(value); self }
    /// Устанавливает значение "radix" в спецификации в `None`
    #[inline]
    pub fn clear_radix(mut self) -> Self { self.radix = None; self }
}

 /**
    Правило "Специальный числовой литерал".
    Ищет токен типа `NumberLiteral` и тестирует его в соответствии с данной спецификацией.

    *   Если свойство спецификации `negative` содержит значение - значение свойства `negative` числа должно совпадать с ним.

    *   Если свойство спецификации `fractional` содержит значение - значение свойства `fractional` числа должно совпадать с ним.

    *   Если свойство спецификации `radix` содержит значение - значение свойства `radix` числа должно входить в его диапазон.
*/
pub fn special_number_literal<'a, 'b>(input: &'a [Token<'b>], spec: NumberLiteralSpec) -> ParserResult<'a, 'b, &'a Token<'b>> {
    match token(input, TokenKindLess::NumberLiteral) {
        IResult::Done(i, output) => {
            match output.kind {
                TokenKind::NumberLiteral {
                    negative, fractional, radix
                } => {
                    if let Some(v) = spec.negative { if v != negative {
                        let desc = if v {
                            "negative number literal"
                        } else {
                            "positive number literal"
                        };
                        return input.err(ParserErrorKind::expected_got_description(
                            desc, TokenKindLess::NumberLiteral, output.text
                        ));
                    } }
                    if let Some(v) = spec.fractional { if v != fractional {
                        let desc = if v {
                            "fractional number literal"
                        } else {
                            "integer number literal"
                        };
                        return input.err(ParserErrorKind::expected_got_description(
                            desc, TokenKindLess::NumberLiteral, output.text
                        ));
                    } }
                    if let Some(v) = spec.radix { if !v.contains(radix) {
                        let desc = format!("number literal with radix between {}..{}", v.start, v.end);
                        return input.err(ParserErrorKind::expected_got_description(
                            desc, TokenKindLess::NumberLiteral, output.text
                        ));
                    } }
                    i.ok(output)
                },
                other => input.err(ParserErrorKind::expected_got_kind(TokenKindLess::NumberLiteral, other.less())),
            }
        },
        other => other,
    }
}

pub const UNSIGNED_INTEGER_SPEC: NumberLiteralSpec = NumberLiteralSpec {
    negative: Some(false),
    fractional: Some(false),
    radix: None,
};

fn make_parse_error(input: &str) -> ParserErrorKind {
    ParserErrorKind::expected_got_description("integer literal", TokenKindLess::NumberLiteral,input)
}

pub fn parse_integer_literal(input: &str) -> Result<u32, ParserErrorKind> {
    let mut chars = input.chars()
        .skip_while(|c| c.is_whitespace());
    let first = match chars.next() {
        Some(v) => v,
        None => return Err(make_parse_error(input)),
    };
    let (
        mut result,
        radix,
    ) = match first {
        '0' => {
            let second = match chars.next() {
                Some(v) => v,
                None => return Err(make_parse_error(input)),
            };
            match second {
                'b' => (0, 2),
                'o' => (0, 8),
                'x' => (0, 16),
                c => match c.to_digit(8) {
                    Some(v) => (v, 8),
                    None => return Err(make_parse_error(input)),
                },
            }
        },
        c => match c.to_digit(10) {
            Some(v) => (v, 10),
            None => return Err(make_parse_error(input)),
        },
    };
    for c in chars {
        match c.to_digit(radix) {
            Some(v) => {
                result *= 10;
                result += v;
            },
            None => return Err(make_parse_error(input)),
        }
    }
    Ok(result)
}

pub fn u32_literal<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, u32> {
    match special_number_literal(input, UNSIGNED_INTEGER_SPEC.clone()) {
        IResult::Done(input, result) => {
            match parse_integer_literal(result.text) {
                Ok(v) => input.ok(v),
                Err(e) => input.err(e),
            }
        }
        IResult::Incomplete(n) => IResult::Incomplete(n),
        IResult::Error(e) => IResult::Error(e),
    }
}

/**
    Правило "Конец ввода".
    Ищет токен типа `EndOfInput` и возвращает его в случае успеха.
*/
pub fn end_of_input<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, &'a Token<'b>> {
    token(input, TokenKindLess::EndOfInput)
}
