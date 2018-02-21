use std::io::{BufRead, stdin, stdout, Write};

use n_transpiler::lexeme_scanner::Scanner;
use n_transpiler::parser_basics::{parse, end_of_input};
use n_transpiler::man_lang::expressions::expression;
use n_transpiler::man_lang::data_sources::data_source;

use super::prettify::Pretty;

pub fn read_line() -> Result<String, i32> {
    stdout().flush().expect("Запись в поток вывода должна быть успешной");
    let input = stdin();
    let mut lock = input.lock();
    let mut result = String::new();
    match lock.read_line(&mut result) {
        Ok(size) => if size == 0 {
            println!("Обнаружен конец ввода. Завершение программы.");
            return Err(0);
        },
        Err(e) => {
            println!("Ошибка чтения ввода: {}.\nЗавершение программы.", e);
            return Err(1);
        }
    };
    Ok(result)
}

parser_rule!(source_or_expression(input) -> Pretty<'source> {
    alt!(input,
        do_parse!(
            source: data_source >>
            end_of_input >>
            (Pretty::Source(source))
        )
        | do_parse!(
            expr: expression >>
            end_of_input >>
            (Pretty::Expression(expr))
        )
    )
});

pub fn do_parse<'source>(input: &'source str) -> Result<Pretty<'source>, String> {
    let tokens = Scanner::scan(input)
        .map_err(|e| format!("Ошибка сканера: {}", e))?;
    let pretty = parse(tokens.as_slice(), source_or_expression)
        .map_err(|e| format!("Ошибка парсера: {}", e))?;
    Ok(pretty)
}