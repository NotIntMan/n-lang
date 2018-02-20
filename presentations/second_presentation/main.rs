use std::io::{ BufRead, stdin, stdout, Write };

#[macro_use]
extern crate nom;
#[macro_use]
extern crate n_transpiler;

use n_transpiler::lexeme_scanner::Scanner;
use n_transpiler::parser_basics::parse;
use n_transpiler::man_lang::expressions::expression;
use n_transpiler::man_lang::data_sources::data_source;

pub mod prettify;

use prettify::Pretty;

parser_rule!(source_or_expression(input) -> Pretty<'source> {
    alt!(input,
        data_source => { |x| Pretty::Source(x) }
        | expression => { |x| Pretty::Expression(x) }
    )
});

fn deal_with_it<'source>(input: &'source str) -> Result<Pretty<'source>, String> {
    let tokens = Scanner::scan(input)
        .map_err(|e| format!("Ошибка сканера: {}", e))?;
    let pretty = parse(tokens.as_slice(), source_or_expression)
        .map_err(|e| format!("Ошибка парсера: {}", e))?;
    Ok(pretty)
}

fn main() {
    println!("Вводите одно за другим, каждое в новой строке.");
    let input = stdin();
    let mut exit_code = 0;
    loop {
        let input = {
            stdout().flush().expect("Запись в поток вывода должна быть успешной");
            let mut lock = input.lock();
            let mut input = String::new();
            match lock.read_line(&mut input) {
                Ok(size) => if size == 0 {
                    println!("Обнаружен конец ввода. Завершение программы.");
                    break
                },
                Err(e) => {
                    exit_code = 1;
                    println!("Ошибка чтения ввода: {}.\nЗавершение программы.", e)
                },
            };
            input
        };
        match deal_with_it(&input) {
            Ok(pretty) => println!("{}", pretty),
            Err(error) => println!("{}", error),
        }
    }
    ::std::process::exit(exit_code);
}
