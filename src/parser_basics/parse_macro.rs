//! Макрос parse! упрощает жизнь во время тестирования

/**
    Во время тестирования приходится часто вызывать код разбора текста в лексемы, а лексем в синтаксические структуры.
    Этот код нельзя скрыть в отдельную функцию т.к. синтаксические структуры могут зависеть от времени жизни массима лексем.

    Однако, в большинстве случаев, это не требуется учитывать. Поэтому, проще написать `parse!("2+2", expression)`, чем
    `parse(Scanner::scan("2+2").expect("Scanner result must be ok").as_slice(), expression)`.

    # Паника

    Не следует использовать этот макрос где-то вне тестов т.к. в случае возникновения ошибки
    лексического или синтаксического анализа код вызовет панику потока.
*/
#[macro_export]
macro_rules! parse {
    ($text: expr, $rule: expr) => {{
        let tokens = $crate::lexeme_scanner::Scanner::scan($text)
            .expect("Scanner result must be ok");
        $crate::parser_basics::parse(tokens.as_slice(), $rule)
            .expect("Parser result must be ok")
    }}
}
