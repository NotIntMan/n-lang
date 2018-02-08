extern crate n_transpiler;

use n_transpiler::lexeme_scanner::{
    Token,
    Scanner,
};
use n_transpiler::parser_basics::parse;
use n_transpiler::desc_lang::primitives::*;
use n_transpiler::desc_lang::compounds::*;

pub fn scan(input: &str) -> Result<Vec<Token>, String> {
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
        result.push_str(&format!("  с аттрибутом {}\n", stringify_attribute(attr)));
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
        result.push_str(&stringify_type(&input.field_type, margin_left + 2));
        result
    } else {
        format!("типа {}", stringify_type(&input.field_type, margin_left + 2))
    }
}

pub fn stringify_type(input: &DataType, margin_left: usize) -> String {
    let margin = generate_margin(margin_left);
    let mut result = String::new();
    match input {
        &DataType::Compound(CompoundDataType::Structure(ref s)) => {
            result.push_str("структура:\n");
            result.push_str(&stringify_attributes(s.attributes.as_slice(), &margin));
            for (name, field) in s.fields.iter() {
                result.push_str(&margin);
                result.push_str(&format!("  с полем {} {}\n", *name, stringify_field(field, margin_left + 4)));
            }
        }
        &DataType::Compound(CompoundDataType::Tuple(ref s)) => {
            result.push_str("кортеж:\n");
            result.push_str(&stringify_attributes(s.attributes.as_slice(), &margin));
            for field in s.fields.iter() {
                result.push_str(&margin);
                result.push_str(&format!("  с полем {}\n", stringify_field(field, margin_left + 4)));
            }
        }
        &DataType::Primitive(PrimitiveDataType::Number(NumberType::Bit { ref size })) => {
            if let &Some(ref s) = size {
                result.push_str(&format!("{} ", *s));
            }
            result.push_str("бит");
        }
        _ => unimplemented!(),
    }
    result
}

pub fn parse_it(input: &[Token]) -> Result<(), String> {
    println!("Передаём токены парсеру.");
    match parse(input, data_type) {
        Ok(_) => Ok(()),
        Err(e) => return Err(format!("Парсер сгенерировал ошибку: {}", e)),
    }
}

fn main() {
    println!("{}", stringify_type(&DataType::Primitive(PrimitiveDataType::Number(NumberType::Bit { size: Some(3) })), 0));
}
