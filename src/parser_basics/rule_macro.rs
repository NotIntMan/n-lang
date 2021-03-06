/**
    Макрос `parser_rule!` является синонимом `named!` из пакета `nom`.
    Только он для текущего проекта.

    ```md
    parser_rule!(
        [pub] $name ($input [, ...$args]) -> $result_type { ...$body }
    )
    ```

    Вот примерное описание синтаксиса макроса.
    Он ожидает увидеть имя объявляемого правила в `$name`,
    имя аргумента, в который будет записан ввод в `$input`,
    тип вовращаемого значения в `$result_type`
    и тело правила в `$body`.

    Правило может быть публичным (`pub`) и иметь аргументы (`...$args`),
    прямо как настоящая функция, но это опциональные возможности.
*/
#[macro_export]
macro_rules! parser_rule {
    (
        $name: ident
        ($input: ident , $($arg_name:ident : $arg_type: ty),*)
        -> $result: ty $body: block
    ) => {
        #[inline]
        fn $name<'token, 'source>(
            $input: &'token [$crate::lexeme_scanner::Token<'source>], $($arg_name: $arg_type),*
        ) -> $crate::nom::IResult<&'token [$crate::lexeme_scanner::Token<'source>], $result, $crate::parser_basics::ParserError<'source>> {
            $body
        }
    };
    (
        $name: ident ($input: ident) -> $result: ty $body: block
    ) => {
        #[inline]
        fn $name<'token, 'source>($input: &'token [$crate::lexeme_scanner::Token<'source>]) ->
        $crate::nom::IResult<&'token [$crate::lexeme_scanner::Token<'source>], $result, $crate::parser_basics::ParserError<'source>> {
            $body
        }
    };
    (
        pub $name: ident
        ($input: ident , $($arg_name:ident : $arg_type: ty),*)
        -> $result: ty $body: block
    ) => {
        #[inline]
        pub fn $name<'token, 'source>(
            $input: &'token [$crate::lexeme_scanner::Token<'source>], $($arg_name: $arg_type),*
        ) -> $crate::nom::IResult<&'token [$crate::lexeme_scanner::Token<'source>], $result, $crate::parser_basics::ParserError<'source>> {
            $body
        }
    };
    (
        pub $name: ident ($input: ident) -> $result: ty $body: block
    ) => {
        #[inline]
        pub fn $name<'token, 'source>($input: &'token [$crate::lexeme_scanner::Token<'source>]) ->
        $crate::nom::IResult<&'token [$crate::lexeme_scanner::Token<'source>], $result, $crate::parser_basics::ParserError<'source>> {
            $body
        }
    };
}
