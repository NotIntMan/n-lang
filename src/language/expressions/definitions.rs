use std::u8::MAX as U8MAX;
//use helpers::IntoStatic;
use helpers::{
    Resolve,
    SyncRef,
};
use lexeme_scanner::ItemPosition;
use helpers::{
    Assertion,
    is_f32_enough,
};
use parser_basics::Identifier;
use language::{
    DataType,
    ItemPath,
    FunctionVariableScope,
    FunctionVariable,
    NumberType,
    PrimitiveDataType,
};
use project_analysis::{
    Item,
    SemanticError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordLiteralType {
    True,
    False,
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LiteralType {
    NumberLiteral {
        negative: bool,
        fractional: bool,
        radix: u32,
        approx_value: f64,
    },
    StringLiteral {
        length: usize,
    },
    BracedExpressionLiteral {
        length: usize,
    },
    KeywordLiteral(KeywordLiteralType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralAST<'source> {
    pub literal_type: LiteralType,
    pub text: Identifier<'source>,
    pub pos: ItemPosition,
}

impl<'source> Assertion for LiteralAST<'source> {
    fn assert(&self, other: &Self) {
        assert_eq!(self.literal_type, other.literal_type);
        assert_eq!(self.text.text(), other.text.text());
    }
}

impl<'source> Into<Literal> for LiteralAST<'source> {
    fn into(self) -> Literal {
        let LiteralAST { literal_type, text, pos } = self;
        Literal {
            literal_type,
            text: text.to_string(),
            pos,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    pub literal_type: LiteralType,
    pub text: String,
    pub pos: ItemPosition,
}

impl Literal {
    pub fn type_of(&self) -> Result<DataType, SemanticError> {
        let result = match self.literal_type {
            LiteralType::NumberLiteral { negative, fractional, radix: _, approx_value } => {
                if fractional {
                    DataType::Primitive(PrimitiveDataType::Number(NumberType::Float {
                        // TODO Изучить вопрос "Зачем float типам точность?"
                        size: None,
                        double: !is_f32_enough(approx_value),
                    }))
                } else {
                    let log2 = if approx_value < 0.0 {
                        (1.0 - approx_value).log2().ceil()
                    } else {
                        (approx_value + 1.0).log2().ceil()
                    };

                    let size = if log2 > 0.0 {
                        if log2 > f64::from(U8MAX) { U8MAX } else { log2 as u8 }
                    } else { 0 };
                    DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
                        size,
                        unsigned: !negative,
                        zerofill: false,
                    }))
                }
            }
        };
        Ok(result)
    }
}

//impl<'source> IntoStatic for Literal<'source> {
//    type Result = Literal<'static>;
//    fn into_static(self) -> Self::Result {
//        let Literal { literal_type, text, pos } = self;
//        Literal {
//            literal_type,
//            text: text.into_static(),
//            pos,
//        }
//    }
//}

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

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionAST<'source> {
    Literal(LiteralAST<'source>),
    Reference(Identifier<'source>),
    BinaryOperation(Box<ExpressionAST<'source>>, BinaryOperator, Box<ExpressionAST<'source>>),
    PrefixUnaryOperation(PrefixUnaryOperator, Box<ExpressionAST<'source>>),
    PostfixUnaryOperation(PostfixUnaryOperator, Box<ExpressionAST<'source>>),
    PropertyAccess(Box<ExpressionAST<'source>>, ItemPath),
    Set(Vec<ExpressionAST<'source>>),
    FunctionCall(ItemPath, Vec<ExpressionAST<'source>>),
}

impl<'source> Assertion for ExpressionAST<'source> {
    fn assert(&self, other: &Self) {
        match self {
            &ExpressionAST::Literal(ref lit_left) => {
                if let &ExpressionAST::Literal(ref lit_right) = other {
                    lit_left.assert(lit_right)
                } else { assert_eq!(self, other) }
            }
            &ExpressionAST::Reference(ref ident_left) => {
                if let &ExpressionAST::Reference(ref ident_right) = other {
                    assert_eq!(ident_left, ident_right);
                } else { assert_eq!(self, other) }
            }
            &ExpressionAST::BinaryOperation(ref left_left, ref left_op, ref left_right) => {
                if let &ExpressionAST::BinaryOperation(ref right_left, ref right_op, ref right_right) = other {
                    (*left_left).assert(&**right_left);
                    assert_eq!(left_op, right_op);
                    (*left_right).assert(&**right_right);
                } else { assert_eq!(self, other) }
            }
            &ExpressionAST::PrefixUnaryOperation(ref left_op, ref left) => {
                if let &ExpressionAST::PrefixUnaryOperation(ref right_op, ref right) = other {
                    assert_eq!(left_op, right_op);
                    (*left).assert(&**right);
                } else { assert_eq!(self, other) }
            }
            &ExpressionAST::PostfixUnaryOperation(ref left_op, ref left) => {
                if let &ExpressionAST::PostfixUnaryOperation(ref right_op, ref right) = other {
                    assert_eq!(left_op, right_op);
                    (*left).assert(&**right);
                } else { assert_eq!(self, other) }
            }
            &ExpressionAST::PropertyAccess(ref left, ref left_path) => {
                if let &ExpressionAST::PropertyAccess(ref right, ref right_path) = other {
                    assert_eq!(left_path.path, right_path.path);
                    (*left).assert(&**right);
                } else { assert_eq!(self, other) }
            }
            &ExpressionAST::Set(ref left_items) => {
                if let &ExpressionAST::Set(ref right_items) = other {
                    left_items.as_slice().assert(&right_items.as_slice());
                } else { assert_eq!(self, other) }
            }
            &ExpressionAST::FunctionCall(ref left_name, ref left_args) => {
                if let &ExpressionAST::FunctionCall(ref right_name, ref right_args) = other {
                    assert_eq!(left_name.path, right_name.path);
                    left_args.as_slice().assert(&right_args.as_slice());
                } else { assert_eq!(self, other) }
            }
        }
    }
}

impl<'source> Assertion<str> for ExpressionAST<'source> {
    fn assert(&self, other: &str) {
        let tokens = ::lexeme_scanner::Scanner::scan(other)
            .expect("Scanner result must be ok");
        let other_expr = ::parser_basics::parse(tokens.as_slice(), super::expression)
            .expect("Parser result must be ok");
        self.assert(&other_expr);
    }
}

impl<'a, 'source> Assertion<&'a str> for ExpressionAST<'source> {
    fn assert(&self, other: &&'a str) {
        self.assert(*other)
    }
}

//impl<'source> IntoStatic for Expression<'source> {
//    type Result = Expression<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            Expression::Literal(value) => Expression::Literal(value.into_static()),
//            Expression::Identifier(value) => Expression::Identifier(value.into_static()),
//            Expression::BinaryOperation(left, op, right) => Expression::BinaryOperation(left.into_static(), op, right.into_static()),
//            Expression::PrefixUnaryOperation(op, value) => Expression::PrefixUnaryOperation(op, value.into_static()),
//            Expression::PostfixUnaryOperation(op, value) => Expression::PostfixUnaryOperation(op, value.into_static()),
//            Expression::PropertyAccess(expr, path) => Expression::PropertyAccess(expr.into_static(), path.into_static()),
//            Expression::Set(value) => Expression::Set(value.into_static()),
//            Expression::FunctionCall(path, args) => Expression::FunctionCall(path.into_static(), args.into_static()),
//        }
//    }
//}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for ExpressionAST<'source> {
    type Result = Expression;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let result = match self {
            &ExpressionAST::Literal(ref lit) => Expression::Literal(lit.clone().into()),
            &ExpressionAST::Reference(ref ident) => {
                let var = scope.access_to_variable(ident.item_pos(), ident.text())
                    .map_err(|e| vec![e])?;
                Expression::Variable(var)
            }
            &ExpressionAST::BinaryOperation(ref left, op, ref right) => {
                let (left, right) = (left, right).resolve(scope)?;
                Expression::BinaryOperation(left, op, right)
            }
            &ExpressionAST::PrefixUnaryOperation(op, ref expr) => Expression::PrefixUnaryOperation(op, expr.resolve(scope)?),
            &ExpressionAST::PostfixUnaryOperation(op, ref expr) => Expression::PostfixUnaryOperation(op, expr.resolve(scope)?),
            &ExpressionAST::PropertyAccess(ref expr, ref path) => {
                let expr = expr.resolve(scope)?;
                let path = path.clone();
                Expression::PropertyAccess(expr, path)
            }
            &ExpressionAST::Set(ref components) => Expression::Set(components.resolve(scope)?),
            &ExpressionAST::FunctionCall(ref function, ref arguments) => {
                let module = scope.context().module();
                let function = match module.get_item(function.path.as_path(), &mut Vec::new()) {
                    // TODO Проверка на правильность ссылки (чтобы нельзя было вызвать не функцию, окда?)
                    Some(item) => item,
                    None => return Err(vec![SemanticError::unresolved_item(
                        function.pos,
                        function.path.clone(),
                    )]),
                };
                // TODO Проверка на правильность типов аргументов
                let arguments = arguments.resolve(scope)?;
                Expression::FunctionCall(function, arguments)
            }
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Variable(SyncRef<FunctionVariable>),
    BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>),
    PrefixUnaryOperation(PrefixUnaryOperator, Box<Expression>),
    PostfixUnaryOperation(PostfixUnaryOperator, Box<Expression>),
    PropertyAccess(Box<Expression>, ItemPath),
    Set(Vec<Expression>),
    FunctionCall(SyncRef<Item>, Vec<Expression>),
}
