//! Набор примитивных шаблонов образования языка

//use syntax_parser::basics::{
//    LexemeCursor,
//    LexemeParser,
//    LexemeParserResult,
//};
//
//use syntax_parser::basics::constants::{
//    ConstSymbol,
//    COMMA,
//    SEMICOLON,
//    CLOSING_BRACES_BRACKET,
//    CLOSING_ROUND_BRACKET,
//    CLOSING_TRIANGULAR_BRACKET,
//    OPENING_BRACES_BRACKET,
//    OPENING_ROUND_BRACKET,
//    OPENING_TRIANGULAR_BRACKET,
//};

use nom::IResult;

use lexeme_scanner::Token;

use super::{
    ParserInput,
    ParserResult,
    symbols,
};

/**
    Шаблон "Подготовка".
    Реализует частичное обратное каррирование парсеров.

    Имеет синтаксис `prepare!( $fn [(...$args)] )`

    Возвращает замыкание единственного аргумента, которое вызовет функцию `$fn`,
    передав ей значение аргумента замыкания, а затем все дополнительные аргументы `...$args`.
    Пустые, ничего не содержащие скобки агрументов могут быть опущены.

    <b>Пример: </b> `prepare!(symbols("+")) <==> |input| symbols(input, "+")`.

    <b>Пример: </b> `prepare!(list(prepare!(symbols("+")), prepare!(symbols(","))))`

    эквивалентно `|input| list(input, |input| symbols(input, "+"), |input| symbols(input, ","))`.
*/
#[macro_export]
macro_rules! prepare {
    ($name:ident) => { |input| $name(input) };
    ($name:ident ()) => { |input| $name(input) };
    ($name:ident ( $( $arg: expr ),+) ) => { |input|
        $name(input, $( $arg ),+)
    };
    ($name:ident ( $( $arg: expr ),+ , ) ) => { |input|
        $name(input, $( $arg ),+)
    };
    ($name:ident! ( $( $arg: tt )+) ) => { |input|
        $name!(input, $( $arg )+)
    };
}

/**
    Шаблон "Список".
    Используется для разбора списка `element`, разделённых `delimiter`.

    В конце списка `delimiter` является опциональным.
    Возвращает вектор успешно разобранных значений (`Vec<ElementOutput>`).
    Никогда не возвращает ошибку.
*/
#[inline]
pub fn list<
    'token,
    'source,
    ElementOutput,
    Element: Fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, ElementOutput>,
    DelimiterOutput,
    Delimiter: Fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, DelimiterOutput>,
>(
    mut input: &'token [Token<'source>], element: Element, delimiter: Delimiter
)
    -> ParserResult<'token, 'source, Vec<ElementOutput>>
    where Token<'source>: 'token
{
    let mut result = Vec::new();
    'parse_cycle: loop {
        match element(input) {
            IResult::Done(new_input, element_result) => {
                input = new_input;
                result.push(element_result);
            },
            _ => { break 'parse_cycle },
        }
        match delimiter(input) {
            IResult::Done(new_input, _) => {
                input = new_input;
            },
            _ => { break 'parse_cycle },
        }
    }
    input.ok(result)
}

#[test]
fn f() {
    use lexeme_scanner::Scanner;
    let buf = Scanner::scan("+, + , +")
        .expect("Scanner result must be ok");
    let pluses = prepare!(list(prepare!(symbols("+")), prepare!(symbols(","))));
    let input = buf.as_slice();
    assert_eq!(
        pluses(input)
            .to_result()
            .expect("Parser result must be ok"),
        vec![(), (), ()]
    );
}

/**
    Шаблон "Обёртка".
    Используется для разбора `element`, который следует после `opening_paren` и до `closing_paren`.
    Частным случаем "обёртки" являются скобки.
*/
#[inline]
pub fn wrap<
    'token,
    'source,
    OpeningParenOutput,
    OpeningParen: Fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, OpeningParenOutput>,
    ElementOutput,
    Element: Fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, ElementOutput>,
    ClosingParenOutput,
    ClosingParen: Fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, ClosingParenOutput>,
>(
    input: &'token [Token<'source>],
    opening_paren: OpeningParen,
    element: Element,
    closing_paren: ClosingParen,
)
    -> ParserResult<'token, 'source, ElementOutput>
    where Token<'source>: 'token
{
    do_parse!(input,
        opening_paren >>
        e: element >>
        closing_paren >>
        (e)
    )
}

/**
    Шаблон "Обёртка символами".
    Используется для разбора `element`, который следует после `opening_paren` и до `closing_paren`.
    Является частным случаем шаблона "Обёртка" и служит для упрощения работы с ней.
*/
#[inline]
pub fn symbol_wrap<
    'token,
    'source,
    ElementOutput,
    Element: Fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, ElementOutput>,
>(
    input: &'token [Token<'source>],
    opening_paren: &str,
    element: Element,
    closing_paren: &str,
)
    -> ParserResult<'token, 'source, ElementOutput>
    where Token<'source>: 'token
{
    wrap(input, prepare!(symbols(opening_paren)), element, prepare!(symbols(closing_paren)))
}

/**
    Шаблон "Обёртка круглыми скобками".
    Используется для разбора `element`, который следует после `(` и до `)`.
    Является частным случаем шаблона "Обёртка символами".
*/
#[inline]
pub fn round_wrap<
    'token,
    'source,
    ElementOutput,
    Element: Fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, ElementOutput>,
>(
    input: &'token [Token<'source>],
    element: Element,
)
    -> ParserResult<'token, 'source, ElementOutput>
    where Token<'source>: 'token
{
    symbol_wrap(input, "(", element, ")")
}

#[test]
fn g() {
    use lexeme_scanner::Scanner;
    let buf = Scanner::scan("(+, +) , +")
        .expect("Scanner result must be ok");
    let pluses = prepare!(list(prepare!(symbols("+")), prepare!(symbols(","))));
    let rounded_pluses = prepare!(round_wrap(pluses));
    let input = buf.as_slice();
    assert_eq!(
        rounded_pluses(input)
            .to_result()
            .expect("Parser result must be ok"),
        vec![(), ()]
    );
}

/**
    Шаблон "Опционально".
    Пытается провести разбор данного парсера, но, в случае неудачи, не возвращает ошибку.

    Является перекрытием макроса opt! из пакета nom с нормальным наследованием типа результата.
*/
#[macro_export]
macro_rules! opt {
    ($i:expr, $mac:ident!( $($args:tt)* )) => {{
        let i_ = $i.clone();
        match $mac!(i_, $($args)*) {
            $crate::nom::IResult::Done(i,o)     => $crate::nom::IResult::Done(i, ::std::option::Option::Some(o)),
            $crate::nom::IResult::Incomplete(i) => $crate::nom::IResult::Incomplete(i),
            _ => $crate::nom::IResult::Done($i, ::std::option::Option::None),
        }
    }};
    ($i:expr, $f:expr) => {
        opt!($i, call!($f));
    };
}
