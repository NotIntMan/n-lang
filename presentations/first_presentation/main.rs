extern crate env_logger;
extern crate n_lang;

use env_logger::try_init;
use std::process::exit;
use std::io::{
    Read,
    stdin,
};

use n_lang::lexeme_scanner::{
    Token,
    Scanner,
};
use n_lang::parser_basics::parse;
use n_lang::syntax_parser::primitive_types::*;
use n_lang::syntax_parser::compound_types::*;

pub fn generate_margin(size: usize) -> String {
    let mut result = String::new();
    for _ in 0..size { result.push(' '); }
    result
}

pub fn stringify_attribute(input: &Attribute) -> String {
    let mut result = String::from(input.name);
    if let &Some(ref args) = &input.arguments {
        if args.len() > 0 {
            result.push_str(" с агргументами: ");
            let mut iter = args.iter();
            if let Some(v) = iter.next() {
                result.push_str(*v);
            }
            for v in iter {
                result.push_str(", ");
                result.push_str(*v);
            }
        } else {
            result.push_str(" с пустым списком аргументов");
        }
    }
    result
}

pub fn stringify_attributes(input: &[Attribute], margin: &str) -> String {
    let mut result = String::new();
    for attr in input {
        result.push_str(margin);
        result.push_str(&format!("с аттрибутом {}\n", stringify_attribute(attr)));
    }
    result
}

pub fn stringify_field(input: &Field, margin_left: usize) -> String {
    if input.attributes.len() > 0 {
        let margin = generate_margin(margin_left);
        let mut result = String::from("\n");
        result.push_str(&stringify_attributes(input.attributes.as_slice(), &margin));
        result.push_str(&margin);
        result.push_str("типа ");
        result.push_str(&stringify_type(&input.field_type, margin_left));
        result
    } else {
        format!("типа {}", stringify_type(&input.field_type, margin_left))
    }
}

pub fn stringify_type(input: &DataType, margin_left: usize) -> String {
    let margin = generate_margin(margin_left);
    let mut result = String::new();
    match input {
        &DataType::Compound(CompoundDataType::Structure(ref s)) => {
            result.push_str("структура");
            for &(ref name, ref field) in s.iter() {
                result.push('\n');
                result.push_str(&margin);
                result.push_str(&format!("с полем {} {}", *name, stringify_field(field, margin_left + 2)));
            }
        }
        &DataType::Compound(CompoundDataType::Tuple(ref s)) => {
            result.push_str("кортеж");
            for field in s.iter() {
                result.push('\n');
                result.push_str(&margin);
                result.push_str(&format!("с полем {}", stringify_field(field, margin_left + 2)));
            }
        }
        &DataType::Primitive(PrimitiveDataType::Number(NumberType::Bit { ref size })) => {
            if let &Some(ref s) = size {
                result.push_str(&format!("{} ", *s));
            }
            result.push_str("бит");
        },
        &DataType::Primitive(PrimitiveDataType::Number(NumberType::Boolean)) => {
            result.push_str("булево число");
        },
        &DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer { ref integer_type, ref unsigned, ref zerofill })) => {
            result.push_str(match integer_type {
                &IntegerType::Tiny => "крохотное ",
                &IntegerType::Small => "маленькое ",
                &IntegerType::Medium => "среднее ",
                &IntegerType::Normal => "",
                &IntegerType::Big => "большое ",
            });
            result.push_str(match unsigned {
                &true => "беззнаковое ",
                &false => "",
            });
            result.push_str(match zerofill {
                &true => "заполняемое нулями ",
                &false => "",
            });
            result.push_str("целое число");
        },
        &DataType::Primitive(PrimitiveDataType::Number(NumberType::Decimal { ref size, ref unsigned, ref zerofill })) => {
            result.push_str(match unsigned {
                &true => "беззнаковое ",
                &false => "",
            });
            result.push_str(match zerofill {
                &true => "заполняемое нулями ",
                &false => "",
            });
            result.push_str("десятичное (с фиксированной точкой) число");
            if let &Some((ref a, ref b)) = size {
                result.push_str(&format!(" с размерностью {}", a));
                if let &Some(ref c) = b {
                    result.push_str(&format!(", из которой {} - дробная часть", c));
                }
            }
        },
        &DataType::Primitive(PrimitiveDataType::Number(NumberType::Float { ref size, ref double })) => {
            result.push_str("число с плавающей точкой");
            if *double {
                result.push_str(" двойной точности");
            }
            if let &Some((ref a, ref b)) = size {
                result.push_str(&format!(" с размерностью {}, из которой {} - дробная часть", a, b));
            }
        },
        &DataType::Primitive(PrimitiveDataType::DateTime(DateTimeType::Date)) => result.push_str("дата"),
        &DataType::Primitive(PrimitiveDataType::DateTime(DateTimeType::Time { ref precision })) => {
            result.push_str("время");
            if let &Some(ref c) = precision {
                result.push_str(&format!(" с точностью миллисекунд {}", c));
            }
        },
        &DataType::Primitive(PrimitiveDataType::DateTime(DateTimeType::Datetime { ref precision })) => {
            result.push_str("дата и время");
            if let &Some(ref c) = precision {
                result.push_str(&format!(" с точностью миллисекунд {}", c));
            }
        },
        &DataType::Primitive(PrimitiveDataType::DateTime(DateTimeType::Timestamp { ref precision })) => {
            result.push_str("временной отпечаток");
            if let &Some(ref c) = precision {
                result.push_str(&format!(" с точностью миллисекунд {}", c));
            }
        },
        &DataType::Primitive(PrimitiveDataType::Year(YearType::Year4)) => result.push_str("год"),
        &DataType::Primitive(PrimitiveDataType::Year(YearType::Year2)) => result.push_str("год с указанием двух последний чисел"),
        &DataType::Primitive(PrimitiveDataType::String(StringType::Varchar { ref size, ref character_set })) => {
            result.push_str("строка");
            if let &Some(ref s) = size {
                result.push_str(&format!(" длиной {}", s));
            }
            result.push_str(match character_set {
                &Some(CharacterSet::Binary) => " в двоичной кодировке",
                &Some(CharacterSet::UTF8) => " в кодировке UTF-8",
                &None => "",
            });
        },
        &DataType::Primitive(PrimitiveDataType::String(StringType::Text { ref character_set })) => {
            result.push_str("текст");
            result.push_str(match character_set {
                &Some(CharacterSet::Binary) => " в двоичной кодировке",
                &Some(CharacterSet::UTF8) => " в кодировке UTF-8",
                &None => "",
            });
        },
        &DataType::Reference(ref refer) => {
            result.push_str("ссылка на тип ");
            let mut path_iter = refer.iter();
            if let Some(path_item) = path_iter.next() {
                result.push_str(*path_item);
            }
            for path_item in path_iter {
                result.push_str("::");
                result.push_str(*path_item);
            }
        },
    }
    result
}

pub fn scan(input: &str) -> Result<Vec<Token>, String> {
    let _ = try_init();
    println!("Начало сканирования. Ввод: {:?}", input);
    let mut result = Vec::new();
    for scanner_iteration_result in Scanner::new(input) {
        let token = match scanner_iteration_result {
            Ok(t) => t,
            Err(e) => return Err(format!("Сканер сгенерировал ошибку: {}", e)),
        };
        println!("Сканер обнаружил токен типа {:?} на {} строке и {} колонке текста", token.kind, token.pos.line, token.pos.column);
        result.push(token);
    }
    println!("Сканирование закончено.");
    Ok(result)
}

pub fn parse_it(input: &[Token]) -> Result<String, String> {
    println!("Передаём токены парсеру.");
    match parse(input, data_type) {
        Ok(t) => Ok(stringify_type(&t, 2)),
        Err(e) => Err(format!("Парсер сгенерировал ошибку: {}", e)),
    }
}

pub fn do_it(input: &str) -> Result<String, String> {
    scan(input).and_then(|tokens| parse_it(tokens.as_slice()))
}

fn main() {
    let stdin = stdin();
    let mut input = String::new();
    match stdin.lock().read_to_string(&mut input) {
        Err(e) => {
            println!("Ошибка чтения: {}", e);
            exit(1);
        },
        _ => {},
    };
    match do_it(&input) {
        Ok(res) => println!("Результат разбора: {}", res),
        Err(err) => {
            println!("{}", err);
            exit(2);
        },
    }
}
