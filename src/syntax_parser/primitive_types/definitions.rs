//! Набор структур для образования грамматики примитивных типов данных

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IntegerType {
    Tiny,
    Small,
    Medium,
    Normal,
    Big,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberType {
    Bit {
        size: Option<u32>,
    },
    Boolean,
    Integer {
        integer_type: IntegerType,
        unsigned: bool,
        zerofill: bool,
    },
    Decimal {
        size: Option<(u32, Option<u32>)>,
        unsigned: bool,
        zerofill: bool,
    },
    Float {
        size: Option<(u32, u32)>,
        double: bool,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DateTimeType {
    Date,
    Time {
        precision: Option<u32>,
    },
    Datetime {
        precision: Option<u32>,
    },
    Timestamp {
        precision: Option<u32>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum YearType {
    Year2,
    Year4,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CharacterSet {
    Binary,
    UTF8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringType {
    Varchar {
        size: Option<u32>,
        character_set: Option<CharacterSet>,
    },
    Text {
        character_set: Option<CharacterSet>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PrimitiveDataType {
    Number(NumberType),
    DateTime(DateTimeType),
    Year(YearType),
    String(StringType),
}
