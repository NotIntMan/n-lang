use std::u8::MAX as U8MAX;
//use helpers::IntoStatic;
use helpers::{
    PathBuf,
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
    CompoundDataType,
    DataType,
    Field,
    ItemPath,
    FunctionVariableScope,
    FunctionVariable,
    NumberType,
    PrimitiveDataType,
    StringType,
};
use project_analysis::{
    Item,
    SemanticItemType,
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
        length: u32,
    },
    BracedExpressionLiteral {
        length: u32,
    },
    KeywordLiteral(KeywordLiteralType),
}

impl LiteralType {
    pub fn type_of(self, pos: ItemPosition) -> Result<DataType, SemanticError> {
        let result = match self {
            LiteralType::NumberLiteral { negative, fractional, radix: _, approx_value } => {
                if fractional {
                    DataType::Primitive(PrimitiveDataType::Number(NumberType::Float {
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
            LiteralType::StringLiteral { length } => {
                let string_type = if length < 256 {
                    StringType::Varchar { size: Some(length), character_set: None }
                } else {
                    StringType::Text { character_set: None }
                };
                DataType::Primitive(PrimitiveDataType::String(string_type))
            }
            LiteralType::BracedExpressionLiteral { length: _ } => {
                return Err(SemanticError::not_supported_yet(pos, "braced expression literals"));
            }
            LiteralType::KeywordLiteral(keyword) => match keyword {
                KeywordLiteralType::True | KeywordLiteralType::False => {
                    DataType::Primitive(PrimitiveDataType::Number(NumberType::Boolean))
                }
                KeywordLiteralType::Null => {
                    return Err(SemanticError::not_supported_yet(pos, "null"));
                }
            }
        };
        Ok(result)
    }
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
pub enum ExpressionASTBody<'source> {
    Literal(LiteralAST<'source>),
    Reference(Identifier<'source>),
    BinaryOperation(Box<ExpressionAST<'source>>, BinaryOperator, Box<ExpressionAST<'source>>),
    PrefixUnaryOperation(PrefixUnaryOperator, Box<ExpressionAST<'source>>),
    PostfixUnaryOperation(PostfixUnaryOperator, Box<ExpressionAST<'source>>),
    PropertyAccess(Box<ExpressionAST<'source>>, ItemPath),
    Set(Vec<ExpressionAST<'source>>),
    FunctionCall(ItemPath, Vec<ExpressionAST<'source>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionAST<'source> {
    pub body: ExpressionASTBody<'source>,
    pub pos: ItemPosition,
}

impl<'source> Assertion for ExpressionAST<'source> {
    fn assert(&self, other: &Self) {
        match &self.body {
            &ExpressionASTBody::Literal(ref lit_left) => {
                if let &ExpressionASTBody::Literal(ref lit_right) = &other.body {
                    lit_left.assert(lit_right)
                } else { assert_eq!(self.body, other.body) }
            }
            &ExpressionASTBody::Reference(ref ident_left) => {
                if let &ExpressionASTBody::Reference(ref ident_right) = &other.body {
                    assert_eq!(ident_left, ident_right);
                } else { assert_eq!(self.body, other.body) }
            }
            &ExpressionASTBody::BinaryOperation(ref left_left, ref left_op, ref left_right) => {
                if let &ExpressionASTBody::BinaryOperation(ref right_left, ref right_op, ref right_right) = &other.body {
                    (*left_left).assert(&**right_left);
                    assert_eq!(left_op, right_op);
                    (*left_right).assert(&**right_right);
                } else { assert_eq!(self.body, other.body) }
            }
            &ExpressionASTBody::PrefixUnaryOperation(ref left_op, ref left) => {
                if let &ExpressionASTBody::PrefixUnaryOperation(ref right_op, ref right) = &other.body {
                    assert_eq!(left_op, right_op);
                    (*left).assert(&**right);
                } else { assert_eq!(self.body, other.body) }
            }
            &ExpressionASTBody::PostfixUnaryOperation(ref left_op, ref left) => {
                if let &ExpressionASTBody::PostfixUnaryOperation(ref right_op, ref right) = &other.body {
                    assert_eq!(left_op, right_op);
                    (*left).assert(&**right);
                } else { assert_eq!(self.body, other.body) }
            }
            &ExpressionASTBody::PropertyAccess(ref left, ref left_path) => {
                if let &ExpressionASTBody::PropertyAccess(ref right, ref right_path) = &other.body {
                    assert_eq!(left_path.path, right_path.path);
                    (*left).assert(&**right);
                } else { assert_eq!(self.body, other.body) }
            }
            &ExpressionASTBody::Set(ref left_items) => {
                if let &ExpressionASTBody::Set(ref right_items) = &other.body {
                    left_items.as_slice().assert(&right_items.as_slice());
                } else { assert_eq!(self.body, other.body) }
            }
            &ExpressionASTBody::FunctionCall(ref left_name, ref left_args) => {
                if let &ExpressionASTBody::FunctionCall(ref right_name, ref right_args) = &other.body {
                    assert_eq!(left_name.path, right_name.path);
                    left_args.as_slice().assert(&right_args.as_slice());
                } else { assert_eq!(self.body, other.body) }
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
        let result = match &self.body {
            &ExpressionASTBody::Literal(ref lit) => {
                let literal: Literal = lit.clone().into();
                let data_type = literal.literal_type.type_of(self.pos)
                    .map_err(|e| vec![e])?;
                Expression {
                    body: ExpressionBody::Literal(literal),
                    pos: self.pos,
                    data_type,
                }
            }
            &ExpressionASTBody::Reference(ref ident) => {
                let var = scope.access_to_variable(ident.item_pos(), ident.text())
                    .map_err(|e| vec![e])?;
                let data_type = var.property_type(&ItemPath {
                    pos: self.pos,
                    path: PathBuf::empty(),
                })
                    .map_err(|e| vec![e])?;
                Expression {
                    body: ExpressionBody::Variable(var),
                    pos: self.pos,
                    data_type,
                }
            }
            &ExpressionASTBody::BinaryOperation(ref left, op, ref right) => {
                let (left, right) = (left, right).resolve(scope)?;
                let data_type = scope.project()
                    .resolve_binary_operation(self.pos, op, &left.data_type, &right.data_type)
                    .map_err(|e| vec![e])?;
                Expression {
                    body: ExpressionBody::BinaryOperation(left, op, right),
                    pos: self.pos,
                    data_type,
                }
            }
            &ExpressionASTBody::PrefixUnaryOperation(op, ref expr) => {
                let expr = expr.resolve(scope)?;
                let data_type = scope.project()
                    .resolve_prefix_unary_operation(self.pos, op, &expr.data_type)
                    .map_err(|e| vec![e])?;
                Expression {
                    body: ExpressionBody::PrefixUnaryOperation(op, expr),
                    pos: self.pos,
                    data_type,
                }
            }
            &ExpressionASTBody::PostfixUnaryOperation(op, ref expr) => {
                let expr = expr.resolve(scope)?;
                let data_type = scope.project()
                    .resolve_postfix_unary_operation(self.pos, op, &expr.data_type)
                    .map_err(|e| vec![e])?;
                Expression {
                    body: ExpressionBody::PostfixUnaryOperation(op, expr),
                    pos: self.pos,
                    data_type,
                }
            }
            &ExpressionASTBody::PropertyAccess(ref expr, ref path) => {
                let expr = expr.resolve(scope)?;
                let data_type = expr.data_type.property_type(self.pos, path.path.as_path())
                    .map_err(|e| vec![e])?;
                let path = path.clone();
                Expression {
                    body: ExpressionBody::PropertyAccess(expr, path),
                    pos: self.pos,
                    data_type,
                }
            }
            &ExpressionASTBody::Set(ref components) => {
                let components = components.resolve(scope)?;
                let fields: Vec<Field> = components.iter()
                    .map(|expr| {
                        let field_type = expr.data_type.clone();
                        Field {
                            attributes: Vec::new(),
                            field_type,
                            position: self.pos,
                        }
                    })
                    .collect();
                let data_type = DataType::Compound(CompoundDataType::Tuple(fields));
                Expression {
                    body: ExpressionBody::Set(components),
                    pos: self.pos,
                    data_type,
                }
            }
            &ExpressionASTBody::FunctionCall(ref function, ref arguments) => {
                let module = scope.context().module();
                let function_item = match module.get_item(function.path.as_path(), &mut Vec::new()) {
                    Some(item) => item,
                    None => return Err(vec![SemanticError::unresolved_item(
                        function.pos,
                        function.path.clone(),
                    )]),
                };
                let arguments: Vec<Expression> = arguments.resolve(scope)?;
                let data_type = {
                    let function_guard = function_item.read();
                    let function = match function_guard.get_function() {
                        Some(func) => func,
                        None => return Err(vec![SemanticError::expected_item_of_another_type(
                            self.pos,
                            SemanticItemType::Function,
                            function_guard.get_type(),
                        )]),
                    };

                    if arguments.len() != function.arguments.len() {
                        return Err(vec![SemanticError::wrong_arguments_count(
                            self.pos,
                            function.arguments.len(),
                            arguments.len(),
                        )]);
                    }

                    for (i, argument) in arguments.iter().enumerate() {
                        let (_, target_data_type) = function.arguments.get_index(i)
                            .expect("The argument can not cease to exist immediately after checking the length of the collection");
                        if !argument.data_type.can_cast(target_data_type) {
                            return Err(vec![SemanticError::cannot_cast_type(
                                self.pos,
                                argument.data_type.clone(),
                                target_data_type.clone(),
                            )]);
                        }
                    }

                    function.result.clone()
                };
                Expression {
                    body: ExpressionBody::FunctionCall(function_item, arguments),
                    pos: self.pos,
                    data_type,
                }
            }
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionBody {
    Literal(Literal),
    Variable(SyncRef<FunctionVariable>),
    BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>),
    PrefixUnaryOperation(PrefixUnaryOperator, Box<Expression>),
    PostfixUnaryOperation(PostfixUnaryOperator, Box<Expression>),
    PropertyAccess(Box<Expression>, ItemPath),
    Set(Vec<Expression>),
    FunctionCall(SyncRef<Item>, Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub body: ExpressionBody,
    pub pos: ItemPosition,
    pub data_type: DataType,
}
