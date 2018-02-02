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
        size: Option<usize>,
    },
    Boolean,
    Integer {
        integer_type: IntegerType,
        unsigned: bool,
        zerofill: bool,
    },
    Decimal {
        size: Option<(usize, Option<usize>)>,
        unsigned: bool,
        zerofill: bool,
    },
    Float {
        size: Option<(usize, usize)>,
        double: bool,
        unsigned: bool,
        zerofill: bool,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DateTimeType {
    Date,
    Time {
        precision: Option<usize>,
    },
    Datetime {
        precision: Option<usize>,
    },
    Timestamp {
        precision: Option<usize>,
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
        size: Option<usize>,
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
