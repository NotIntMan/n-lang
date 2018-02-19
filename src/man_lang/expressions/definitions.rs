use lexeme_scanner::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeywordLiteralType {
    True,
    False,
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralType {
    NumberLiteral {
        negative: bool,
        fractional: bool,
        radix: u32,
    },
    StringLiteral {
        length: usize,
    },
    BracedExpressionLiteral {
        length: usize,
    },
    KeywordLiteral(KeywordLiteralType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal<'source> {
    pub literal_type: LiteralType,
    pub token: Token<'source>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Logical operators
    Or,
    XOr,
    And,
    // Bit operators
    BitOr,
    BitXOr,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    // Set operators
    IsIn,
    // Comparison operators
    Equals,
    MoreThanOrEquals,
    MoreThan,
    LessThanOrEquals,
    LessThan,
    Like,
    SoundsLike,
    RegExp,
    // Arithmetic operators
    Plus,
    Minus,
    Times,
    Divide,
    Mod,
    Div,
    Pow,
    // Language specific operators
    // TODO Collate, как мне кажется, не вписвается в модель бинарных операций. Может, переместить его в декларации и высказывания? Или в унарные выражения?
    Collate,
    Interval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixUnaryOperator {
    // Logical operators
    Not,
    // Set operators
    All,
    Any,
    // Arithmetic operators
    Plus,
    Minus,
    Tilde,
    // Language specific operators
    Binary,
    Row,
    Exists,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostfixUnaryOperator {
    // Primitive comparison operators
    IsNull,
    IsTrue,
    IsFalse,
    IsUnknown,
//    FieldAppeal(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression<'source> {
    Literal(Literal<'source>),
    Identifier(Token<'source>),
    BinaryOperation(Box<Expression<'source>>, BinaryOperator, Box<Expression<'source>>),
    PrefixUnaryOperation(PrefixUnaryOperator, Box<Expression<'source>>),
    PostfixUnaryOperation(PostfixUnaryOperator, Box<Expression<'source>>),
    PropertyAccess(Box<Expression<'source>>, Vec<&'source str>),
    Set(Vec<Expression<'source>>),
    FunctionCall(&'source str, Vec<Expression<'source>>),
}
