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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberTypeLess {
    Bit,
    Boolean,
    Integer,
    Decimal,
    Float,
}

impl NumberType {
    pub fn less(&self) -> NumberTypeLess {
        match self {
            &NumberType::Bit {
                size: _,
            } => NumberTypeLess::Bit,
            &NumberType::Boolean => NumberTypeLess::Boolean,
            &NumberType::Integer {
                integer_type: _,
                unsigned: _,
                zerofill: _,
            } => NumberTypeLess::Integer,
            &NumberType::Decimal {
                size: _,
                unsigned: _,
                zerofill: _,
            } => NumberTypeLess::Decimal,
            &NumberType::Float {
                size: _,
                double: _,
                unsigned: _,
                zerofill: _,
            } => NumberTypeLess::Float,
        }
    }
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
pub enum DateTimeTypeLess {
    Date,
    Time,
    Datetime,
    Timestamp,
}

impl DateTimeType {
    pub fn less(&self) -> DateTimeTypeLess {
        match self {
            &DateTimeType::Date => DateTimeTypeLess::Date,
            &DateTimeType::Time {
                precision: _,
            } => DateTimeTypeLess::Time,
            &DateTimeType::Datetime {
                precision: _,
            } => DateTimeTypeLess::Datetime,
            &DateTimeType::Timestamp {
                precision: _,
            } => DateTimeTypeLess::Timestamp,
        }
    }
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
pub enum StringTypeLess {
    Varchar,
    Text,
}

impl StringType {
    pub fn less(&self) -> StringTypeLess {
        match self {
            &StringType::Varchar {
                size: _,
                character_set: _,
            } => StringTypeLess::Varchar,
            &StringType::Text {
                character_set: _,
            } => StringTypeLess::Text,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PrimitiveDataType {
    Number(NumberType),
    DateTime(DateTimeType),
    Year(YearType),
    String(StringType),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PrimitiveDataTypeLess {
    Number,
    DateTime,
    Year,
    String,
}

impl PrimitiveDataType {
    pub fn less(&self) -> PrimitiveDataTypeLess {
        match self {
            &PrimitiveDataType::Number(_) => PrimitiveDataTypeLess::Number,
            &PrimitiveDataType::DateTime(_) => PrimitiveDataTypeLess::DateTime,
            &PrimitiveDataType::Year(_) => PrimitiveDataTypeLess::Year,
            &PrimitiveDataType::String(_) => PrimitiveDataTypeLess::String,
        }
    }
}
