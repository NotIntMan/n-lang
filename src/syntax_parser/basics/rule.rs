//! Примитивы синтаксического анализа, определённые над правилами распознавания лексем

use helpers::num_range::NumRange;

use super::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
};

/**
    Примитив "Возможно".
    Выполняет разбор хранимого правила и, в случае успеха, возвращает его результат, обёрнутый в `Some(_)`.
    В случае неудачи откатывает положение курсора и возвращает `None`.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct RuleOption<R>(pub R);

impl<'a, 'b, R: LexemeParser<'a, 'b>> LexemeParser<'a, 'b> for RuleOption<R> {
    type Result = Option<R::Result>;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let begin = cursor.index;
        let result = self.0.parse(cursor);
        match result {
            Ok(v) => { Ok(Some(v)) }
            Err(_) => {
                cursor.index = begin;
                Ok(None)
            }
        }
    }
}

/**
    Макрос parse_option! является обобщением примитива "Возможно".
    Служит для упрощения написания логики реализуемого правила.

    # Синтаксис

    Использовать его возможно единственным образом:
    ```
    parse_option!(
        cursor: $cursor, $rule
    )
    ```
    Здесь, в качестве `$cursor` указывается курсор буфера, по которому будет шагать разбор.
    Это нужно для сохранения позиции начала разбора.
    В качестве `$rule` указывается правило, которое предстоит обернуть.

    <b>Важно:</b>
    В качестве `$rule` стоит указывать именно <i>выражение</i>, а не имя правила.
    Это нужно для реализации комбинации макросов-помощников.
*/
#[allow(unused_macros)]
#[macro_export]
macro_rules! parse_option {
    (
        cursor: $cursor: expr,
        $rule: expr
    ) => {{
        let __begin = $cursor.index;
        match $rule {
            Ok(r) => Ok(Some(r)),
            Err(_) => {
                $cursor.index = __begin;
                Ok(None)
            },
       }
    }};
}

/**
    Примитив "Ветвление".
    Выполняет разбор сначала первого правила и, в случае неудачи,
    откатывает положение курсора и выполняет разбор второго правила,
    результат которого и возвращает, вне зависимости от успешности.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBranch<A, B>(pub A, pub B);

impl<'a, 'b, A, B, R> LexemeParser<'a, 'b> for RuleBranch<A, B>
    where A: LexemeParser<'a, 'b, Result=R>,
          B: LexemeParser<'a, 'b, Result=R>,
{
    type Result = R;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let begin = cursor.index;
        let mut err = match self.0.parse(cursor) {
            Ok(v) => return Ok(v),
            Err(e) => e,
        };
        cursor.index = begin;
        match self.1.parse(cursor) {
            Ok(v) => Ok(v),
            Err(e) => {
                err.append(e);
                Err(err)
            }
        }
    }
}

/**
    Макрос parse_branch! является обобщением примитива "Ветвление".
    Служит для упрощения написания логики реализуемого правила.

    # Синтаксис

    Использовать его рекомендуется единственным образом:
    ```
    parse_branch!(
        cursor: $cursor,
        $( $rule ),+
    )
    ```
    Здесь, в качестве `$cursor` указывается курсор буфера, по которому будет шагать разбор.
    Это нужно для сохранения позиции начала разбора.
    В качестве `$rule` через запятую перечисляются выражения разбора, которые предстоит обернуть.

    <b>Важно:</b>
    В качестве `$rule` стоит указывать именно <i>выражение</i>, а не имя правила.
    Это нужно для реализации комбинации макросов-помощников.
*/
#[allow(unused_macros)]
#[macro_export]
macro_rules! parse_branch {
    (
        begin: $begin: expr,
        cursor: $cursor: expr,
        error: $error: expr,
        $first_rule: expr,
        $( $rule: expr ),+
    ) => {
        match $first_rule {
            Err(e) => {
                $error.append(e);
                $cursor.index = $begin;
                parse_branch!(
                    begin: $begin,
                    cursor: $cursor,
                    error: $error,
                    $( $rule ),+
                )
            },
            ok => ok,
        }
    };
    (
        begin: $begin: expr,
        cursor: $cursor: expr,
        error: $error: expr,
        $rule: expr
    ) => {
        $rule
    };
    (
        cursor: $cursor: expr,
        $first_rule: expr,
        $( $rule: expr ),+
    ) => {{
        let __begin = $cursor.index;
        match $first_rule {
            Err(mut __error) => {
                parse_branch!(
                    begin: __begin,
                    cursor: $cursor,
                    error: __error,
                    $( $rule ),+
                )
            },
            ok => ok,
        }
    }};
    (
        cursor: $cursor: expr,
        $( $rule: expr ),+
        ,
    ) => {
        parse_branch!(
            cursor: $cursor,
            $( $rule ),+
        )
    };
}

/**
    Примитив "Повторение".
    Выполняет повторный разбор правила до тех пор,
    пока не достигнет верхней границы хранимого диапазона,
    которой, кстати, может и не быть,
    или пока не встретит ошибку.

    При обнаружении ошибки, выполняет проверку количества успешно разобранных итераций.
    Если количество входит в хранимый диапазон, возвращает вектор полученных результатов,
    предватирельно откатив положение курсора на начало последней итерации.
    Если количество успешно разобранных итераций не входит в хранимый диапазон,
    возвращает обнаруженную ошибку.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct RuleRepeat<A, B>(pub A, pub B);

use std::fmt::Debug;

impl<'a, 'b, A, B> LexemeParser<'a, 'b> for RuleRepeat<A, B>
    where A: LexemeParser<'a, 'b> + Debug,
          A::Result: Debug,
          B: NumRange<usize>,
{
    type Result = Vec<A::Result>;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let mut result = match self.1.get_max() {
            Some(end) => Vec::with_capacity(end),
            None => Vec::new(),
        };
        let max = self.1.get_max();
        'parsing_cycle: for i in 1.. {
            if match max {
                Some(m) => i >= m,
                None => false,
            } { break 'parsing_cycle; }
            let begin = cursor.index;
            match self.0.parse(cursor) {
                Ok(v) => {
                    result.push(v)
                }
                Err(e) => {
                    if self.1.is_contains(&result.len()) {
                        cursor.index = begin;
                        break 'parsing_cycle;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(result)
    }
}

/**
    Макрос parse_repeat! является обобщением примитива "Повторение".
    Служит для упрощения написания логики реализуемого правила.

    # Синтаксис

    Использовать его возможно единственным образом:
    ```
    parse_repeat!(
        cursor: $cursor, $rule, $range
    )
    ```
    Здесь, в качестве `$cursor` указывается курсор буфера, по которому будет шагать разбор.
    Это нужно для сохранения позиции начала разбора.
    В качестве `$rule` указывается правило, которое предстоит обернуть.
    В качестве `$range` указывается промежуток значений, либо конкретное число ожидаемых повторений.

    <b>Важно:</b>
    В качестве `$rule` стоит указывать именно <i>выражение</i>, а не имя правила.
    Это нужно для реализации комбинации макросов-помощников.
*/
#[allow(unused_macros)]
#[macro_export]
macro_rules! parse_repeat {
    (
        cursor: $cursor: expr,
        range: $range: expr,
        $rule: expr
    ) => {{
        let __range = $range;
        let __max = NumRange::get_max(&__range);
        let mut __result = match __max {
            Some(end) => Vec::with_capacity(end),
            None => Vec::new(),
        };
        let mut __error = None;
        '__parsing_cycle: for __i in 1.. {
            if match __max {
                Some(__m) => __i >= __m,
                None => false,
            } { break '__parsing_cycle; }
            let __begin = $cursor.index;
            match $rule {
                Ok(__v) => {
                    __result.push(__v)
                }
                Err(__e) => {
                    if NumRange::is_contains(&__range, &__result.len()) {
                        $cursor.index = __begin;
                        break '__parsing_cycle;
                    } else {
                        __error = Some(__e);
                        break '__parsing_cycle;
                    }
                }
            }
        }
        match __error {
            Some(__e) => Err(__e),
            None => Ok(__result),
        }
    }};
}

/**
    Макрос parse_sequence! не является обобщением, но так же служит для помощи в реализации грамматики.

    # Синтаксис

    Использовать его рекомендуется единственным образом:
    ```
    parse_sequence!(
        $( $rule ),+
        return $result
    )
    ```
    В качестве `$rule` через запятую перечисляются выражения разбора, которые предстоит обернуть.
    В качестве `$resolve` указывается код, который будет выполнен в случае успеха.

    <b>Стоит учесть:</b>
    Для использования результата, который сгенерировало правило, стоит начать его с имени и знака равно.
    Например, `left = number.parse(...)`.

    <b>Важно:</b>
    В качестве `$rule` стоит указывать именно <i>выражение</i>, а не имя правила.
    Это нужно для реализации комбинации макросов-помощников.
*/
#[allow(unused_macros)]
#[macro_export]
macro_rules! parse_sequence {
    (
        return $result: expr
    ) => { $result };
    (
        $first_rule: expr,
        $( $other: tt )+
    ) => {
        match $first_rule {
            Ok(_) => parse_sequence!( $( $other )+ ),
            Err(e) => Err(e),
        }
    };
    (
        let $var: pat =
        $first_rule: expr,
        $( $other: tt )+
    ) => {
        match $first_rule {
            Ok($var) => parse_sequence!( $( $other )+ ),
            Err(e) => Err(e),
        }
    };
}
