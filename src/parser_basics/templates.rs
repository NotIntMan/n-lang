//! Набор примитивных шаблонов образования языка

// TODO Попробовать запилить шаблоны, которые сочетаются с nom

use lexeme_scanner::Token;
use nom::IResult;
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
    mut input: &'token [Token<'source>], element: Element, delimiter: Delimiter,
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
            }
            _ => { break 'parse_cycle; }
        }
        match delimiter(input) {
            IResult::Done(new_input, _) => {
                input = new_input;
            }
            _ => { break 'parse_cycle; }
        }
    }
    input.ok(result)
}

/**
    Шаблон "Список через запяную".
    Используется для разбора списка `element`, разделённых `,`.

    Является частным случаем шаблона "Список".
*/
#[inline]
pub fn comma_list<
    'token,
    'source,
    ElementOutput,
    Element: Fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, ElementOutput>,
>(
    input: &'token [Token<'source>], element: Element,
)
    -> ParserResult<'token, 'source, Vec<ElementOutput>>
    where Token<'source>: 'token
{
    list(input, element, prepare!(symbols(",")))
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
    opening_paren: &'source str,
    element: Element,
    closing_paren: &'source str,
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

/// Шаблон "Список через запятую, обёрнутый круглыми скобками".
#[inline]
pub fn rounded_comma_list<
    'token,
    'source,
    ElementOutput,
    Element: Fn(&'token [Token<'source>]) -> ParserResult<'token, 'source, ElementOutput>,
>(
    input: &'token [Token<'source>],
    element: Element,
)
    -> ParserResult<'token, 'source, Vec<ElementOutput>>
    where Token<'source>: 'token
{
    do_parse!(input,
        apply!(symbols, "(") >>
        x: apply!(comma_list, element) >>
        apply!(symbols, ")") >>
        (x)
    )
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

/**
    Шаблон "Альтернатива".
    Пытается провести разбор правила. В слчае неудачи, идёт к следующему.
    В случае полной неудачи, возвращает группу ошибок.

    Является перекрытием макроса alt! из пакета nom с нормальным группированием ошибок.
*/
#[macro_export]
macro_rules! alt {
    (__impl $i: expr, $rule: ident! ( $($args:tt)* ) => { $gen:expr }) => {{
        #[cfg(feature="parser_trace")]
        trace!("alt! macro goes into {} rule", stringify!($rule!($i, $($args)*)));
        match $rule!($i, $($args)*) {
            $crate::nom::IResult::Done(i, o) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got success result and pass it into closure");
                $crate::nom::IResult::Done(i, $gen(o))
            },
            $crate::nom::IResult::Incomplete(n) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got incomplete status and return it");
                $crate::nom::IResult::Incomplete(n)
            },
            $crate::nom::IResult::Error(e) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got error and return it {:?}", e);
                $crate::nom::IResult::Error(e)
            },
        }
    }};
    (__impl $i: expr, $rule: ident! ( $($args:tt)* )) => {{
        #[cfg(feature="parser_trace")]
        trace!("alt! macro goes into {} rule", stringify!($rule!($i, $($args)*)));
        match $rule!($i, $($args)*) {
            $crate::nom::IResult::Done(i, o) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got success result and return it");
                $crate::nom::IResult::Done(i, o)
            },
            $crate::nom::IResult::Incomplete(n) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got incomplete status and return it");
                $crate::nom::IResult::Incomplete(n)
            },
            $crate::nom::IResult::Error(e) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got error and return it {:?}", e);
                $crate::nom::IResult::Error(e)
            },
        }
    }};
    (__impl $i: expr, $rule: ident! ( $($args:tt)* ) => { $gen:expr } | $($rest: tt)+) => {{
        #[cfg(feature="parser_trace")]
        trace!("alt! macro goes into {} rule", stringify!($rule!($i, $($args)*)));
        match $rule!($i, $($args)*) {
            $crate::nom::IResult::Done(i, o) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got success result and pass it into closure");
                $crate::nom::IResult::Done(i, $gen(o))
            },
            $crate::nom::IResult::Incomplete(n) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got incomplete status and return it");
                $crate::nom::IResult::Incomplete(n)
            },
            $crate::nom::IResult::Error($crate::nom::ErrorKind::Custom(mut e)) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got error and tries to parse another rule. Error: {}", e);
                match alt!(__impl $i, $($rest)+) {
                    $crate::nom::IResult::Done(i, o) => $crate::nom::IResult::Done(i, o),
                    $crate::nom::IResult::Incomplete(n) => $crate::nom::IResult::Incomplete(n),
                    $crate::nom::IResult::Error($crate::nom::ErrorKind::Custom(f)) => {
                        e.append_group(f);
                        $crate::nom::IResult::Error($crate::nom::ErrorKind::Custom(e))
                    },
                    $crate::nom::IResult::Error(other) => $crate::nom::IResult::Error(other),
                }
            },
            $crate::nom::IResult::Error(other) => $crate::nom::IResult::Error(other),
        }
    }};
    (__impl $i: expr, $rule: ident! ( $($args:tt)* ) | $($rest: tt)+) => {{
        #[cfg(feature="parser_trace")]
        trace!("alt! macro goes into {} rule", stringify!($rule!($i, $($args)*)));
        match $rule!($i, $($args)*) {
            $crate::nom::IResult::Done(i, o) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got success result and pass it into closure");
                $crate::nom::IResult::Done(i, o)
            },
            $crate::nom::IResult::Incomplete(n) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got incomplete status and return it");
                $crate::nom::IResult::Incomplete(n)
            },
            $crate::nom::IResult::Error($crate::nom::ErrorKind::Custom(mut e)) => {
                #[cfg(feature="parser_trace")]
                trace!("alt! macro got error and tries to parse another rule. Error: {}", e);
                match alt!(__impl $i, $($rest)+) {
                    $crate::nom::IResult::Done(i, o) => $crate::nom::IResult::Done(i, o),
                    $crate::nom::IResult::Incomplete(n) => $crate::nom::IResult::Incomplete(n),
                    $crate::nom::IResult::Error($crate::nom::ErrorKind::Custom(f)) => {
                        e.append_group(f);
                        $crate::nom::IResult::Error($crate::nom::ErrorKind::Custom(e))
                    },
                    $crate::nom::IResult::Error(other) => $crate::nom::IResult::Error(other),
                }
            },
            $crate::nom::IResult::Error(other) => $crate::nom::IResult::Error(other),
        }
    }};
    (__impl $i: expr, $rule: ident $($rest: tt)*) => { alt!(__impl $i, call!($rule) $($rest)*) };
    ($i: expr, $($rest :tt)+) => {{
        #[cfg(feature="parser_trace")]
        trace!("alt! macro is started parsing");
        alt!(__impl $i, $($rest)+)
    }};
}
