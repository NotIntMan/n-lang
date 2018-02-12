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
    // Language specific operators
    Collate,
    Interval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression<'source> {
    Literal(Literal<'source>),
    Identifier(Token<'source>),
    BinaryOperation(Box<Expression<'source>>, BinaryOperator, Box<Expression<'source>>),
}
