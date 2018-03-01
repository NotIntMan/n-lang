use n_transpiler::man_lang::data_sources::{
    DataSource,
    JoinCondition,
    JoinType,
};
use n_transpiler::man_lang::expressions::{
    BinaryOperator,
    Expression,
    LiteralType,
    KeywordLiteralType,
    PostfixUnaryOperator,
    PrefixUnaryOperator,
};

fn pretty_join_type(t: JoinType) -> &'static str {
    match t {
        JoinType::Cross => "объединение",
        JoinType::Left => "левостороннее объединение",
        JoinType::Right => "правостороннее объединение",
        JoinType::Full => "полное объединение",
    }
}

fn pretty_join_condition(c: &Option<JoinCondition>, padding: usize) -> String {
    let d = match c {
        &Some(ref x) => x,
        &None => return format!("без условия"),
    };
    match d {
        &JoinCondition::Expression(ref expr) =>
            format!("по выражению: {}", pretty_expression(expr, padding + 2)),
        &JoinCondition::Using(ref path_list) =>
            format!("по эквивалентности полей:{}", pretty_property_path_list(path_list, padding + 2)),
        &JoinCondition::Natural => format!("по эквивалентности всех полей таблиц"),
    }
}

fn pretty_literal_type(l: &LiteralType) -> String {
    match l {
        &LiteralType::NumberLiteral { negative, fractional, radix } => {
            let n = if negative { "отрицательное" } else { "положительное" };
            let f = if fractional { "дробное" } else { "целое" };
            format!("{} {} число в системе счисления с базисом {}", n, f, radix)
        }
        &LiteralType::StringLiteral { length } =>
            format!("строка длиной {}", length),
        &LiteralType::BracedExpressionLiteral { length } =>
            format!("выражение в опострофах длиной {}", length),
        &LiteralType::KeywordLiteral(KeywordLiteralType::True) => format!("true"),
        &LiteralType::KeywordLiteral(KeywordLiteralType::False) => format!("false"),
        &LiteralType::KeywordLiteral(KeywordLiteralType::Null) => format!("null"),
    }
}

fn pretty_binary_operator(op: BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::Or => "или",
        BinaryOperator::XOr => "исключающее или",
        BinaryOperator::And => "и",
        BinaryOperator::BitOr => "битовое или",
        BinaryOperator::BitXOr => "битовое исключающее или",
        BinaryOperator::BitAnd => "битовое и",
        BinaryOperator::ShiftLeft => "сдвиг влево",
        BinaryOperator::ShiftRight => "сдвиг вправо",
        BinaryOperator::IsIn => "принадлежность",
        BinaryOperator::Equals => "эквивалентность",
        BinaryOperator::MoreThanOrEquals => "более чем, либо равно",
        BinaryOperator::MoreThan => "более чем",
        BinaryOperator::LessThanOrEquals => "менее чем, либо равно",
        BinaryOperator::LessThan => "мене чем",
        BinaryOperator::Like => "сравнение с шаблоном",
        BinaryOperator::SoundsLike => "сравнение с шаблоном по звучанию",
        BinaryOperator::RegExp => "тест регулярным выражением",
        BinaryOperator::Plus => "сложение",
        BinaryOperator::Minus => "вычитание",
        BinaryOperator::Times => "умножение",
        BinaryOperator::Divide => "деление",
        BinaryOperator::Mod => "остаток от деления",
        BinaryOperator::Div => "частное деления",
        BinaryOperator::Pow => "возведение в степень",
        BinaryOperator::Interval => "создание промежутка",
    }
}

fn new_line<S: ToString>(target: S, padding: usize) -> String {
    let mut result = String::new();
    result.reserve(padding + 1);
    result.push('\n');
    for _ in 0..padding {
        result.push(' ');
    }
    result.push_str(&target.to_string());
    result
}

fn pretty_postfix_unary_operator(op: PostfixUnaryOperator) -> &'static str {
    match op {
        PostfixUnaryOperator::IsNull => "является ли это null",
        PostfixUnaryOperator::IsTrue => "является ли это true",
        PostfixUnaryOperator::IsFalse => "является ли это false",
        PostfixUnaryOperator::IsUnknown => "является ли это неизвестным",
    }
}

fn pretty_prefix_unary_operator(op: PrefixUnaryOperator) -> &'static str {
    match op {
        PrefixUnaryOperator::Not => "не",
        PrefixUnaryOperator::All => "всё",
        PrefixUnaryOperator::Any => "хотя бы один",
        PrefixUnaryOperator::Plus => "нейтральный плюс",
        PrefixUnaryOperator::Minus => "унарный минус",
        PrefixUnaryOperator::Tilde => "тильда",
        PrefixUnaryOperator::Binary => "получение двоичных данных",
        PrefixUnaryOperator::Row => "получение сырых данных",
        PrefixUnaryOperator::Exists => "существование",
    }
}

fn pretty_property_path(path: &Vec<&str>) -> String {
    let mut result = String::with_capacity(path.len() * 10);
    let mut iter = path.iter();
    if let Some(s) = iter.next() {
        result.push_str(*s);
    }
    for s in iter {
        result.push_str(".");
        result.push_str(*s);
    }
    result
}

fn pretty_module_path(path: &Vec<&str>) -> String {
    let mut result = String::with_capacity(path.len() * 10);
    let mut iter = path.iter();
    if let Some(s) = iter.next() {
        result.push_str(*s);
    }
    for s in iter {
        result.push_str("::");
        result.push_str(*s);
    }
    result
}

fn pretty_property_path_list(list: &Vec<Vec<&str>>, padding: usize) -> String {
    let mut result = String::with_capacity(list.len() * 128);
    for (i, expr) in list.iter().enumerate() {
        let line = new_line(format!("#{} - {}", i, pretty_property_path(expr)), padding);
        result.push_str(&line);
    }
    result
}

fn pretty_expression_list(list: &Vec<Expression>, padding: usize) -> String {
    let mut result = String::with_capacity(list.len() * 128);
    for (i, expr) in list.iter().enumerate() {
        let line = new_line(format!("#{} - {}", i, pretty_expression(expr, padding + 2)), padding);
        result.push_str(&line);
    }
    result
}

fn pretty_expression(expr: &Expression, padding: usize) -> String {
    match expr {
        &Expression::Literal(ref literal) => {
            format!("литерал типа {} с текстом {:?}", pretty_literal_type(&literal.literal_type), literal.token.text)
        }
        &Expression::Identifier(token) => {
            format!("идентификатор с текстом {:?}", token.text)
        }
        &Expression::BinaryOperation(ref left, operator, ref right) => {
            format!("бинарная операция: {}{}{}",
                    pretty_binary_operator(operator),
                    new_line(format!("слева: {}", pretty_expression(&*left, padding + 2)), padding),
                    new_line(format!("справа: {}", pretty_expression(&*right, padding + 2)), padding),
            )
        }
        &Expression::PostfixUnaryOperation(operator, ref expr) => {
            format!("префиксная унарная операция: {}{}",
                    pretty_postfix_unary_operator(operator),
                    new_line(pretty_expression(&*expr, padding + 2), padding),
            )
        }
        &Expression::PrefixUnaryOperation(operator, ref expr) => {
            format!("префиксная унарная операция: {}{}",
                    pretty_prefix_unary_operator(operator),
                    new_line(pretty_expression(&*expr, padding + 2), padding),
            )
        }
        &Expression::PropertyAccess(ref expr, ref path) => {
            format!("доступ к свойству{}{}",
                    new_line(format!("у выражения: {}", pretty_expression(&*expr, padding + 2)), padding),
                    new_line(format!("путь до свойства: {}", pretty_property_path(path)), padding),
            )
        }
        &Expression::Set(ref expressions) => {
            if expressions.is_empty() {
                format!("пустой набор")
            } else {
                format!("набор значений:{}", pretty_expression_list(expressions, padding + 2))
            }
        }
        &Expression::FunctionCall(ref name, ref arguments) => {
            if arguments.is_empty() {
                format!("вызов функции {} без аргументов", pretty_module_path(name))
            } else {
                format!("вызов функции {} с аргументами:{}", pretty_module_path(name), pretty_expression_list(arguments, padding + 2))
            }
        }
    }
}

fn pretty_data_source(source: &DataSource, padding: usize) -> String {
    match source {
        &DataSource::Table { ref name, alias } => match alias {
            Some(alias) => format!("таблица {} (с синонимом {})", pretty_module_path(name), alias),
            None => format!("таблица {}", pretty_module_path(name)),
        },
        &DataSource::Join { join_type, ref condition, ref left, ref right } => {
            format!("{}{}{}{}",
                    pretty_join_type(join_type),
                    new_line(pretty_join_condition(condition, padding + 2), padding),
                    new_line(format!("слева: {}", pretty_data_source(&*left, padding + 2)), padding),
                    new_line(format!("справа: {}", pretty_data_source(&*right, padding + 2)), padding),
            )
        }
        &DataSource::Selection { query: _, alias: _ } => format!("выборка из базы данных (данные опущены т.к. не имеют отношения к теме презентации)"),
    }
}

pub enum Pretty<'source> {
    Source(DataSource<'source>),
    Expression(Expression<'source>),
}

use std::fmt::{
    Display,
    Formatter,
    Result as FResult,
};

impl<'source> Display for Pretty<'source> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            &Pretty::Source(ref source) => write!(f, "Источник данных: {}", pretty_data_source(source, 2)),
            &Pretty::Expression(ref expr) => write!(f, "Выражение: {}", pretty_expression(expr, 2)),
        }
    }
}
