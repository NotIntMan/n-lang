#[macro_export]
macro_rules! parser_rule {
    (
        $name: ident
        ($input: ident , $($arg_name:ident : $arg_type: ty),*)
        -> $result: ty $body: block
    ) => {
        fn $name<'token, 'source>(
            $input: &'token [Token<'source>], $($arg_name: $arg_type),*
        ) -> nom::IResult<&'token [Token<'source>], $result, parser_basics::ParserError> {
            $body
        }
    };
    (
        $name: ident ($input: ident) -> $result: ty $body: block
    ) => {
        fn $name<'token, 'source>($input: &'token [Token<'source>]) ->
        nom::IResult<&'token [Token<'source>], $result, parser_basics::ParserError> {
            $body
        }
    };
    (
        pub $name: ident
        ($input: ident , $($arg_name:ident : $arg_type: ty),*)
        -> $result: ty $body: block
    ) => {
        pub fn $name<'token, 'source>(
            $input: &'token [Token<'source>], $($arg_name: $arg_type),*
        ) -> nom::IResult<&'token [Token<'source>], $result, parser_basics::ParserError> {
            $body
        }
    };
    (
        pub $name: ident ($input: ident) -> $result: ty $body: block
    ) => {
        pub fn $name<'token, 'source>($input: &'token [Token<'source>]) ->
        nom::IResult<&'token [Token<'source>], $result, parser_basics::ParserError> {
            $body
        }
    };
}
