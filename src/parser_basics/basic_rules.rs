//! Набор примитивных правил для образования языка

use std::ops::Range;
use std::borrow::Cow;
use std::fmt;
use nom::IResult;
use helpers::into_static::IntoStatic;
use lexeme_scanner::{
    ItemPosition,
    Token,
    TokenKind,
    TokenKindLess,
    SymbolPosition,
};
use super::{
    ParserErrorKind,
    ParserErrorTokenInfo,
    ParserInput,
    ParserResult,
    token,
    exact_token,
    some_token,
};

/**
    Правило "Ничего".
    Ничего не ожидает, ничего не возвращает.
    Ошибкам взяться неоткуда.
*/
pub fn none<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, ()> {
    input.ok(())
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'source>(pub Cow<'source, str>);

pub type StaticIdentifier = Identifier<'static>;

impl<'source> Identifier<'source> {
    pub fn new(text: &'source str) -> Self {
        Identifier(Cow::Borrowed(text))
    }
    pub fn get_borrowed_text(&self) -> Option<&'source str> {
        match &self.0 {
            &Cow::Borrowed(borrow) => Some(borrow),
            &Cow::Owned(_) => None,
        }
    }
    pub fn get_text(&self) -> &str {
        match &self.0 {
            &Cow::Borrowed(borrow) => borrow,
            &Cow::Owned(ref own) => &own[..],
        }
    }
}

impl<'source> IntoStatic for Identifier<'source> {
    type Result = StaticIdentifier;
    fn into_static(self) -> Self::Result {
        match self {
            Identifier(Cow::Borrowed(borrow)) => Identifier(Cow::Owned(borrow.to_string())),
            Identifier(Cow::Owned(own)) => Identifier(Cow::Owned(own)),
        }
    }
}

impl<'source> PartialEq<str> for Identifier<'source> {
    fn eq(&self, other: &str) -> bool {
        &*self.0 == other
    }

    fn ne(&self, other: &str) -> bool {
        &*self.0 != other
    }
}

impl<'source, 'target> PartialEq<&'target str> for Identifier<'source> {
    fn eq(&self, other: &&'target str) -> bool {
        self.get_text() == *other
    }

    fn ne(&self, other: &&'target str) -> bool {
        self.get_text() != *other
    }
}

impl<'source, 'target> PartialEq<Identifier<'source>> for &'target str {
    fn eq(&self, other: &Identifier<'source>) -> bool {
        *self == other.get_text()
    }

    fn ne(&self, other: &Identifier<'source>) -> bool {
        *self != other.get_text()
    }
}

impl<'source> fmt::Debug for Identifier<'source> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let item = self.get_text();
        if f.alternate() {
            write!(f, "Identifier: {:#?}", &*item)
        } else {
            write!(f, "Identifier: {:?}", &*item)
        }
    }
}

/**
    Правило "Идентификатор".
    Ищет токен типа `Word` и возвращает его текст в случае успеха.
*/
pub fn identifier<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Identifier<'source>> {
    token(input, TokenKindLess::Word)
        .map(|token| Identifier::new(token.text))
}

/**
    Правило "Сырой идентификатор".
    Ищет токен типа `Word` и возвращает ссылку на него в случае успеха.
*/
pub fn identifier_raw<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, &'token Token<'source>> {
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
    "set",
    "join",
    "value",
    "values",
);

/**
    Правило "Не ключевой идентификатор".
    Ищет токен типа `Word` и возвращает ссылку на него в случае успеха.
    Если текст токена содержится в списке ключевых слов `KEYWORD_LIST`, возвращает ошибку `ExpectedGot`.
*/
pub fn not_keyword_identifier<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Identifier<'source>> {
    match identifier(input) {
        IResult::Done(new_input, result) => {
            let text = result.get_borrowed_text()
                .expect("Rule \"identifier\" should not return owned text as result");
            if KEYWORD_LIST.contains(&text) {
                input.err(ParserErrorKind::expected_got(
                    ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::Word, "not keyword identifier"),
                    ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::Word, text),
                ))
            } else {
                new_input.ok(result)
            }
        }
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
pub fn keyword<'token, 'source>(input: &'token [Token<'source>], expected_text: &'source str) -> ParserResult<'token, 'source, &'token Token<'source>> {
    match token(input, TokenKindLess::Word) {
        IResult::Done(i, output) => {
            if !compare_words(output.text, expected_text) {
                return input.err(ParserErrorKind::expected_got(
                    ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::Word, expected_text),
                    ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::Word, output.text),
                ));
            }
            i.ok(output)
        }
        IResult::Incomplete(n) => IResult::Incomplete(n),
        IResult::Error(e) => IResult::Error(e),
    }
}

/**
    Правило "Числовой литерал".
    Ищет токен типа `NumberLiteral` и возвращает его в случае успеха.
*/
pub fn number_literal<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, &'token Token<'source>> {
    token(input, TokenKindLess::NumberLiteral)
}

/**
    Правило "Строковый литерал".
    Ищет токен типа `StringLiteral` и возвращает его в случае успеха.
*/
pub fn string_literal<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, &'token Token<'source>> {
    token(input, TokenKindLess::StringLiteral)
}

/**
    Правило "Литерал выражения".
    Ищет токен типа `BracedExpressionLiteral` и возвращает его в случае успеха.
*/
pub fn braced_expression_literal<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, &'token Token<'source>> {
    token(input, TokenKindLess::BracedExpressionLiteral)
}

/**
    Правило "Специальные символы".
    Ищет токен типа `SymbolGroup` с текстом, эквивалентным данному.
    Ничего не возвращает в случае успеха.
*/
pub fn symbols<'token, 'source>(input: &'token [Token<'source>], text: &'source str) -> ParserResult<'token, 'source, ()> {
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
    pub fn set_negative(mut self, value: bool) -> Self {
        self.negative = Some(value);
        self
    }
    /// Устанавливает значение "negative" в спецификации в `None`
    #[inline]
    pub fn clear_negative(mut self) -> Self {
        self.negative = None;
        self
    }
    /// Устанавливает значение "fractional" в спецификации в `Some(value)`
    #[inline]
    pub fn set_fractional(mut self, value: bool) -> Self {
        self.fractional = Some(value);
        self
    }
    /// Устанавливает значение "fractional" в спецификации в `None`
    #[inline]
    pub fn clear_fractional(mut self) -> Self {
        self.fractional = None;
        self
    }
    /// Устанавливает значение "radix" в спецификации в `Some(value)`
    #[inline]
    pub fn set_radix(mut self, value: Range<u32>) -> Self {
        self.radix = Some(value);
        self
    }
    /// Устанавливает значение "radix" в спецификации в `None`
    #[inline]
    pub fn clear_radix(mut self) -> Self {
        self.radix = None;
        self
    }
}

/**
   Правило "Специальный числовой литерал".
   Ищет токен типа `NumberLiteral` и тестирует его в соответствии с данной спецификацией.

   *   Если свойство спецификации `negative` содержит значение - значение свойства `negative` числа должно совпадать с ним.

   *   Если свойство спецификации `fractional` содержит значение - значение свойства `fractional` числа должно совпадать с ним.

   *   Если свойство спецификации `radix` содержит значение - значение свойства `radix` числа должно входить в его диапазон.
*/
pub fn special_number_literal<'token, 'source>(input: &'token [Token<'source>], spec: NumberLiteralSpec) -> ParserResult<'token, 'source, &'token Token<'source>> {
    match token(input, TokenKindLess::NumberLiteral) {
        IResult::Done(i, output) => {
            match output.kind {
                TokenKind::NumberLiteral {
                    negative, fractional, radix
                } => {
                    if let Some(v) = spec.negative {
                        if v != negative {
                            let desc = if v {
                                "negative number literal"
                            } else {
                                "positive number literal"
                            };
                            return input.err(ParserErrorKind::expected_got(
                                ParserErrorTokenInfo::from_desc(desc),
                                ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::NumberLiteral, output.text),
                            ));
                        }
                    }
                    if let Some(v) = spec.fractional {
                        if v != fractional {
                            let desc = if v {
                                "fractional number literal"
                            } else {
                                "integer number literal"
                            };
                            return input.err(ParserErrorKind::expected_got(
                                ParserErrorTokenInfo::from_desc(desc),
                                ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::NumberLiteral, output.text),
                            ));
                        }
                    }
                    if let Some(v) = spec.radix {
                        if !v.contains(radix) {
                            return input.err(ParserErrorKind::expected_got(
                                ParserErrorTokenInfo::from_desc("number literal with a specific radix"),
                                ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::NumberLiteral, output.text),
                            ));
                        }
                    }
                    i.ok(output)
                }
                other => input.err(ParserErrorKind::expected_got(
                    ParserErrorTokenInfo::from_kind(TokenKindLess::NumberLiteral),
                    ParserErrorTokenInfo::from_kind(other.less()),
                )),
            }
        }
        other => other,
    }
}

/// Спецификация для правила "Специальный числовой литерал".
/// Описывает целочисленный неотрицательный числовой литерал в любой системе исчисления.
/// Используется правилом "u32-литерал".
pub const UNSIGNED_INTEGER_SPEC: NumberLiteralSpec = NumberLiteralSpec {
    negative: Some(false),
    fractional: Some(false),
    radix: None,
};

/// Генерирует ошибку, которая "ожидала" целочисленный неотрицательный числовой литерал, а получила другой числовой литерал.
fn make_parse_error(input: &str) -> ParserErrorKind {
    ParserErrorKind::expected_got(
        ParserErrorTokenInfo::from_desc("integer literal"),
        ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::NumberLiteral, input),
    )
}

/// Совершает попытку разбора целочисленного неотрицательного литерала.
/// Полностью соответствует спецификации числовых литералов языка.
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
        }
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
            }
            None => return Err(make_parse_error(input)),
        }
    }
    Ok(result)
}

/**
    Правило "u32-литерал".
    Ищет целочисленных неотрицательный литерал, проводит его разбор и возвращает значение.
    В случае неудачи возвращает ошибку.
*/
pub fn u32_literal<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, u32> {
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
pub fn end_of_input<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, &'token Token<'source>> {
    token(input, TokenKindLess::EndOfInput)
}

/**
    Правило "Позиция символа".
    Возвращает положение первого встречного токена. В случае неудачи, возвращает ошибку типа `UnexpectedEnd`.
*/
pub fn symbol_position<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, SymbolPosition> {
    some_token(input).map(|t: &Token| t.pos)
}

/**
    Правило "Позиция элемента".
    Получает начало положения, затем ищет положение первого встречного токена и возвращает положение элемента. В случае неудачи, возвращает ошибку типа `UnexpectedEnd`.
*/
pub fn item_position<'token, 'source>(input: &'token [Token<'source>], begin: SymbolPosition) -> ParserResult<'token, 'source, ItemPosition> {
    some_token(input).map(|t: &Token| {
        let end = t.pos;
        if begin > end {
            ItemPosition { begin: end, end: begin }
        } else {
            ItemPosition { begin, end }
        }
    })
}
