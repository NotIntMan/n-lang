//! Парсеры, распознающие грамматику примитивных типов данных

use lexeme_scanner::Token;

use parser_basics::{
    keyword,
    none,
    ParserResult,
    round_wrap,
    symbols,
    u32_literal,
};
use language::{
    CharacterSet,
    DateTimeType,
    NumberType,
    PrimitiveDataType,
    StringType,
    YearType,
};

/// [(u32)]
parser_rule!(single_size(i) -> Option<u32> {
    opt!(i, apply!(round_wrap, u32_literal))
});

/// "bit" single_size
parser_rule!(bit(i) -> NumberType {
    do_parse!(i,
        apply!(keyword, "bit") >>
        size: single_size >>
        (NumberType::Bit { size })
    )
});

/// ["unsigned"] ["zerofill"]
parser_rule!(unsigned_zerofill(i) -> (bool, bool) {
    do_parse!(i,
        unsigned: opt!(apply!(keyword, "unsigned")) >>
        zerofill: opt!(apply!(keyword, "zerofill")) >>
        ((unsigned.is_some(), zerofill.is_some()))
    )
});

/// unsigned_zerofill ["tiny" | "small" | "medium" | "big"] "integer"
parser_rule!(integer(i) -> NumberType {
    do_parse!(i,
        u: unsigned_zerofill >>
        size: alt!(
            apply!(keyword, "tiny") => { |_| 8 } |
            apply!(keyword, "small") => { |_| 16 } |
            apply!(keyword, "medium") => { |_| 24 } |
            apply!(keyword, "big") => { |_| 64 } |
            none => { |_| 32 }
        ) >>
        apply!(keyword, "integer") >>
        ({
            let (unsigned, zerofill) = u;
            NumberType::Integer{ size, unsigned, zerofill }
        })
    )
});

/// unsigned_zerofill "decimal" [(u32[, u32])]
parser_rule!(decimal(i) -> NumberType {
    do_parse!(i,
        u: unsigned_zerofill >>
        apply!(keyword, "decimal") >>
        size: opt!(apply!(round_wrap, prepare!(do_parse!(
            a: u32_literal >>
            b: opt!(do_parse!(
                apply!(symbols, ",") >>
                x: u32_literal >>
                (x)
            )) >>
            ((a, b))
        )))) >>
        ({
            let (unsigned, zerofill) = u;
            NumberType::Decimal { size, unsigned, zerofill }
        })
    )
});

/// [(u32, u32)]
parser_rule!(float_size(i) -> Option<(u32, u32)> {
    opt!(i, apply!(round_wrap, prepare!(do_parse!(
        a: u32_literal >>
        apply!(symbols, ",") >>
        b: u32_literal >>
        ((a, b))
    ))))
});

/// unsigned_zerofill "float" float_size
parser_rule!(float(i) -> NumberType {
    do_parse!(i,
        apply!(keyword, "float") >>
        size: float_size >>
        (NumberType::Float { size, double: false })
    )
});

/// unsigned_zerofill "double" float_size
parser_rule!(double(i) -> NumberType {
    do_parse!(i,
        apply!(keyword, "double") >>
        size: float_size >>
        (NumberType::Float { size, double: true })
    )
});

/// bit | "boolean" | integer | decimal | float | double
parser_rule!(number_type(i) -> NumberType {
    alt!(i,
        bit |
        apply!(keyword, "boolean") => { |_| NumberType::Boolean } |
        integer |
        decimal |
        float |
        double
    )
});

/// $word single_size
parser_rule!(datetime_precision(i, word: &'source str) -> Option<u32> {
    do_parse!(i,
        apply!(keyword, word) >>
        s: single_size >>
        (s)
    )
});

/// "date" | "time" single_size | "datetime" single_size | "timestamp" single_size
parser_rule!(datetime_type(i) -> DateTimeType {
    alt!(i,
        apply!(keyword, "date") => { |_| DateTimeType::Date } |
        apply!(datetime_precision, "time") => { |precision| DateTimeType::Time { precision } } |
        apply!(datetime_precision, "datetime") => { |precision| DateTimeType::Datetime { precision } } |
        apply!(datetime_precision, "timestamp") => { |precision| DateTimeType::Timestamp { precision } }
    )
});

/// "year4" | "year2" | "year"
parser_rule!(year_type(i) -> YearType {
    alt!(i,
        apply!(keyword, "year4") => { |_| YearType::Year4 } |
        apply!(keyword, "year2") => { |_| YearType::Year2 } |
        apply!(keyword, "year") => { |_| YearType::Year4 }
    )
});

/// "binary" | "utf8"
parser_rule!(character_set_type(i) -> CharacterSet {
    alt!(i,
        apply!(keyword, "binary") => { |_| CharacterSet::Binary } |
        apply!(keyword, "utf8") => { |_| CharacterSet::UTF8 }
    )
});

/// "character" "set" character_set_type
parser_rule!(character_set(i) -> CharacterSet {
    do_parse!(i,
        apply!(keyword, "character") >>
        apply!(keyword, "set") >>
        s: character_set_type >>
        (s)
    )
});

/// "varchar" single_size [character_set] | "text" [character_set]
parser_rule!(string_type(i) -> StringType {
    alt!(i,
        do_parse!(
            apply!(keyword, "varchar") >>
            size: single_size >>
            character_set: opt!(character_set) >>
            (StringType::Varchar { size, character_set })
        ) |
        do_parse!(
            apply!(keyword, "text") >>
            character_set: opt!(character_set) >>
            (StringType::Text { character_set })
        )
    )
});

/// Парсер, реализующий разбор грамматики примитивных типов
pub fn primitive_data_type<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, PrimitiveDataType> {
    alt!(input,
        number_type => { |x| PrimitiveDataType::Number(x) } |
        datetime_type => { |x| PrimitiveDataType::DateTime(x) } |
        year_type => { |x| PrimitiveDataType::Year(x) } |
        string_type => { |x| PrimitiveDataType::String(x) }
    )
}
