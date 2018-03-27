use lexeme_scanner::Token;
use helpers::assertion::Assertion;
use syntax_parser::others::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordLiteralType {
    True,
    False,
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl<'source> Assertion for Literal<'source> {
    fn assert(&self, other: &Self) {
        assert_eq!(self.literal_type, other.literal_type);
        self.token.assert(&other.token);
    }
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
    IsNull,
    IsTrue,
    IsFalse,
    IsUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression<'source> {
    Literal(Literal<'source>),
    Identifier(Token<'source>),
    BinaryOperation(Box<Expression<'source>>, BinaryOperator, Box<Expression<'source>>),
    PrefixUnaryOperation(PrefixUnaryOperator, Box<Expression<'source>>),
    PostfixUnaryOperation(PostfixUnaryOperator, Box<Expression<'source>>),
    PropertyAccess(Box<Expression<'source>>, Path<'source>),
    Set(Vec<Expression<'source>>),
    FunctionCall(Path<'source>, Vec<Expression<'source>>),
}

impl<'source> Assertion for Expression<'source> {
    fn assert(&self, other: &Self) {
        match self {
            &Expression::Literal(ref lit_left) => {
                if let &Expression::Literal(ref lit_right) = other {
                    lit_left.assert(lit_right)
                } else { assert_eq!(self, other) }
            }
            &Expression::Identifier(ref token_left) => {
                if let &Expression::Identifier(ref token_right) = other {
                    token_left.assert(token_right)
                } else { assert_eq!(self, other) }
            }
            &Expression::BinaryOperation(ref left_left, ref left_op, ref left_right) => {
                if let &Expression::BinaryOperation(ref right_left, ref right_op, ref right_right) = other {
                    (*left_left).assert(&**right_left);
                    assert_eq!(left_op, right_op);
                    (*left_right).assert(&**right_right);
                } else { assert_eq!(self, other) }
            }
            &Expression::PrefixUnaryOperation(ref left_op, ref left) => {
                if let &Expression::PrefixUnaryOperation(ref right_op, ref right) = other {
                    assert_eq!(left_op, right_op);
                    (*left).assert(&**right);
                } else { assert_eq!(self, other) }
            }
            &Expression::PostfixUnaryOperation(ref left_op, ref left) => {
                if let &Expression::PostfixUnaryOperation(ref right_op, ref right) = other {
                    assert_eq!(left_op, right_op);
                    (*left).assert(&**right);
                } else { assert_eq!(self, other) }
            }
            &Expression::PropertyAccess(ref left, ref left_path) => {
                if let &Expression::PropertyAccess(ref right, ref right_path) = other {
                    assert_eq!(left_path, right_path);
                    (*left).assert(&**right);
                } else { assert_eq!(self, other) }
            }
            &Expression::Set(ref left_items) => {
                if let &Expression::Set(ref right_items) = other {
                    left_items.as_slice().assert(&right_items.as_slice());
                } else { assert_eq!(self, other) }
            }
            &Expression::FunctionCall(ref left_name, ref left_args) => {
                if let &Expression::FunctionCall(ref right_name, ref right_args) = other {
                    assert_eq!(left_name, right_name);
                    left_args.as_slice().assert(&right_args.as_slice());
                } else { assert_eq!(self, other) }
            }
        }
    }
}

impl<'source> Assertion<str> for Expression<'source> {
    fn assert(&self, other: &str) {
        let tokens = ::lexeme_scanner::Scanner::scan(other)
            .expect("Scanner result must be ok");
        let other_expr = ::parser_basics::parse(tokens.as_slice(),super::expression)
            .expect("Parser result must be ok");
        self.assert(&other_expr);
    }
}

impl<'a, 'source> Assertion<&'a str> for Expression<'source> {
    fn assert(&self, other: &&'a str) {
        self.assert(*other)
    }
}
