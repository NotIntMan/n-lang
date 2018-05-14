use std::u8::MAX as U8MAX;
use std::fmt;
use std::cmp;
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
    NumberType,
    PrimitiveDataType,
    StringType,
};
use project_analysis::{
    Item,
    FunctionVariableScope,
    FunctionVariable,
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
                        if log2 >= f64::from(U8MAX) {
                            U8MAX
                        } else {
                            let log2 = log2 as u8;
                            if negative { log2 + 1 } else { log2 }
                        }
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

impl BinaryOperator {
    pub fn get_description(&self) -> &'static str {
        match *self {
            BinaryOperator::Or => "or",
            BinaryOperator::XOr => "exclusive or",
            BinaryOperator::And => "and",
            BinaryOperator::BitOr => "bit or",
            BinaryOperator::BitXOr => "bit exclusive or",
            BinaryOperator::BitAnd => "bit and",
            BinaryOperator::ShiftLeft => "shift left",
            BinaryOperator::ShiftRight => "shift right",
            BinaryOperator::IsIn => "is in",
            BinaryOperator::Equals => "equals",
            BinaryOperator::MoreThanOrEquals => "more than or equals",
            BinaryOperator::MoreThan => "more than",
            BinaryOperator::LessThanOrEquals => "less than or equals",
            BinaryOperator::LessThan => "less than",
            BinaryOperator::Like => "like",
            BinaryOperator::SoundsLike => "sounds like",
            BinaryOperator::RegExp => "reg exp",
            BinaryOperator::Plus => "plus",
            BinaryOperator::Minus => "minus",
            BinaryOperator::Times => "times",
            BinaryOperator::Divide => "divide",
            BinaryOperator::Mod => "mod",
            BinaryOperator::Div => "div",
            BinaryOperator::Pow => "pow",
            BinaryOperator::Interval => "interval",
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_description())
    }
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

impl PrefixUnaryOperator {
    pub fn get_description(&self) -> &'static str {
        match *self {
            PrefixUnaryOperator::Not => "not",
            PrefixUnaryOperator::All => "all",
            PrefixUnaryOperator::Any => "any",
            PrefixUnaryOperator::Plus => "plus",
            PrefixUnaryOperator::Minus => "minus",
            PrefixUnaryOperator::Tilde => "tilde",
            PrefixUnaryOperator::Binary => "binary",
            PrefixUnaryOperator::Row => "row",
            PrefixUnaryOperator::Exists => "exists",
        }
    }
}

impl fmt::Display for PrefixUnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_description())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostfixUnaryOperator {
    IsNull,
    IsTrue,
    IsFalse,
    IsUnknown,
}

impl PostfixUnaryOperator {
    pub fn get_description(&self) -> &'static str {
        match *self {
            PostfixUnaryOperator::IsNull => "is null",
            PostfixUnaryOperator::IsTrue => "is true",
            PostfixUnaryOperator::IsFalse => "is false",
            PostfixUnaryOperator::IsUnknown => "is unknown",
        }
    }
}

impl fmt::Display for PostfixUnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_description())
    }
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
        match &self.body {
            &ExpressionASTBody::Literal(ref lit) => {
                Expression::literal(self.pos, lit)
                    .map_err(|e| vec![e])
            }
            &ExpressionASTBody::Reference(ref ident) => {
                Expression::variable(scope, self.pos, ident)
                    .map_err(|e| vec![e])
            }
            &ExpressionASTBody::BinaryOperation(ref left, op, ref right) => {
                Expression::binary_operation(scope, self.pos, op, left, right)
            }
            &ExpressionASTBody::PostfixUnaryOperation(op, ref expr) => {
                Expression::postfix_unary_operation(scope, self.pos, op, expr)
            }
            &ExpressionASTBody::PrefixUnaryOperation(op, ref expr) => {
                Expression::prefix_unary_operation(scope, self.pos, op, expr)
            }
            &ExpressionASTBody::PropertyAccess(ref expr, ref path) => {
                Expression::property_access(scope, self.pos, expr, path)
            }
            &ExpressionASTBody::Set(ref components) => {
                Expression::set(scope, self.pos, components)
            }
            &ExpressionASTBody::FunctionCall(ref function, ref arguments) => {
                Expression::function_call(scope, self.pos, function, arguments)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionBody {
    Literal(Literal),
    Variable(SyncRef<FunctionVariable>),
    BinaryOperation(Box<Expression>, BinaryOperator, Box<Expression>),
    PostfixUnaryOperation(PostfixUnaryOperator, Box<Expression>),
    PrefixUnaryOperation(PrefixUnaryOperator, Box<Expression>),
    PropertyAccess(Box<Expression>, ItemPath),
    Set(Vec<Expression>),
    FunctionCall(SyncRef<Item>, Vec<Expression>),
    StdFunctionCall(String, Vec<Expression>),
}

impl ExpressionBody {
    pub fn can_be_selected_by_aggregation_query(&self, aggregates: &Vec<Expression>) -> bool {
        match self {
            ExpressionBody::Literal(_) => true,
            ExpressionBody::Variable(_) => false,
            ExpressionBody::BinaryOperation(left, _, right) => {
                left.can_be_selected_by_aggregation_query(aggregates)
                    && right.can_be_selected_by_aggregation_query(aggregates)
            }
            ExpressionBody::PostfixUnaryOperation(_, expr) => {
                expr.can_be_selected_by_aggregation_query(aggregates)
            }
            ExpressionBody::PrefixUnaryOperation(_, expr) => {
                expr.can_be_selected_by_aggregation_query(aggregates)
            }
            ExpressionBody::PropertyAccess(expr, _) => {
                expr.can_be_selected_by_aggregation_query(aggregates)
            }
            ExpressionBody::Set(expressions) => {
                expressions.iter()
                    .all(|expr| expr.can_be_selected_by_aggregation_query(aggregates))
            }
            ExpressionBody::FunctionCall(_, expressions) => {
                expressions.iter()
                    .all(|expr| expr.can_be_selected_by_aggregation_query(aggregates))
            }
            ExpressionBody::StdFunctionCall(_, expressions) => {
                expressions.iter()
                    .all(|expr| expr.can_be_selected_by_aggregation_query(aggregates))
            }
        }
    }
}

impl cmp::PartialEq for ExpressionBody {
    fn eq(&self, other: &ExpressionBody) -> bool {
        match &self {
            ExpressionBody::Literal(lit) => {
                if let ExpressionBody::Literal(other_lit) = other {
                    return lit == other_lit;
                }
            }
            ExpressionBody::Variable(var) => {
                if let ExpressionBody::Variable(other_var) = other {
                    return var.is_same_ref(other_var);
                }
            }
            ExpressionBody::BinaryOperation(left, op, right) => {
                if let ExpressionBody::BinaryOperation(other_left, other_op, other_right) = other {
                    return (op == other_op) && left.eq(other_left) && right.eq(other_right);
                }
            }
            ExpressionBody::PostfixUnaryOperation(op, expr) => {
                if let ExpressionBody::PostfixUnaryOperation(other_op, other_expr) = other {
                    return (op == other_op) && expr.eq(other_expr);
                }
            }
            ExpressionBody::PrefixUnaryOperation(op, expr) => {
                if let ExpressionBody::PrefixUnaryOperation(other_op, other_expr) = other {
                    return (op == other_op) && expr.eq(other_expr);
                }
            }
            ExpressionBody::PropertyAccess(expr, prop) => {
                if let ExpressionBody::PropertyAccess(other_expr, other_prop) = other {
                    return (prop.path == other_prop.path) && expr.eq(other_expr);
                }
            }
            ExpressionBody::Set(expressions) => {
                if let ExpressionBody::Set(other_expressions) = other {
                    return expressions == other_expressions;
                }
            }
            ExpressionBody::FunctionCall(function, arguments) => {
                if let ExpressionBody::FunctionCall(other_function, other_arguments) = other {
                    return function.is_same_ref(other_function)
                        &&
                        (arguments == other_arguments);
                }
            }
            ExpressionBody::StdFunctionCall(function, arguments) => {
                if let ExpressionBody::StdFunctionCall(other_function, other_arguments) = other {
                    return (function == other_function)
                        &&
                        (arguments == other_arguments);
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub body: ExpressionBody,
    pub pos: ItemPosition,
    pub data_type: DataType,
}

impl Expression {
    pub fn literal(pos: ItemPosition, lit: &LiteralAST) -> Result<Self, SemanticError> {
        let literal: Literal = lit.clone().into();
        let data_type = literal.literal_type.type_of(pos)?;
        Ok(Expression {
            body: ExpressionBody::Literal(literal),
            pos,
            data_type,
        })
    }
    pub fn variable(
        scope: &SyncRef<FunctionVariableScope>,
        pos: ItemPosition,
        ident: &Identifier,
    ) -> Result<Self, SemanticError> {
        let var = scope.access_to_variable(ident.item_pos(), ident.text())?;
        let data_type = var.property_type(&ItemPath { pos, path: PathBuf::empty() })?;
        Ok(Expression {
            body: ExpressionBody::Variable(var),
            pos,
            data_type,
        })
    }
    pub fn binary_operation(
        scope: &SyncRef<FunctionVariableScope>,
        pos: ItemPosition,
        op: BinaryOperator,
        left: &Box<ExpressionAST>,
        right: &Box<ExpressionAST>,
    ) -> Result<Self, Vec<SemanticError>>
    {
        let (left, right) = (left, right).resolve(scope)?;
        let data_type = scope.project()
            .resolve_binary_operation(pos, op, &left.data_type, &right.data_type)
            .map_err(|e| vec![e])?
            .output;
        Ok(Expression {
            body: ExpressionBody::BinaryOperation(left, op, right),
            pos,
            data_type,
        })
    }
    pub fn postfix_unary_operation(
        scope: &SyncRef<FunctionVariableScope>,
        pos: ItemPosition,
        op: PostfixUnaryOperator,
        expr: &Box<ExpressionAST>,
    ) -> Result<Self, Vec<SemanticError>>
    {
        let expr = expr.resolve(scope)?;
        let data_type = scope.project()
            .resolve_postfix_unary_operation(pos, op, &expr.data_type)
            .map_err(|e| vec![e])?
            .output;
        Ok(Expression {
            body: ExpressionBody::PostfixUnaryOperation(op, expr),
            pos,
            data_type,
        })
    }
    pub fn prefix_unary_operation(
        scope: &SyncRef<FunctionVariableScope>,
        pos: ItemPosition,
        op: PrefixUnaryOperator,
        expr: &Box<ExpressionAST>,
    ) -> Result<Self, Vec<SemanticError>>
    {
        let expr = expr.resolve(scope)?;
        let data_type = scope.project()
            .resolve_prefix_unary_operation(pos, op, &expr.data_type)
            .map_err(|e| vec![e])?
            .output;
        Ok(Expression {
            body: ExpressionBody::PrefixUnaryOperation(op, expr),
            pos,
            data_type,
        })
    }
    pub fn property_access(
        scope: &SyncRef<FunctionVariableScope>,
        pos: ItemPosition,
        expr: &Box<ExpressionAST>,
        path: &ItemPath,
    ) -> Result<Self, Vec<SemanticError>>
    {
        let expr = expr.resolve(scope)?;
        let data_type = expr.data_type.property_type(pos, path.path.as_path())
            .map_err(|e| vec![e])?;
        let path = path.clone();
        Ok(Expression {
            body: ExpressionBody::PropertyAccess(expr, path),
            pos,
            data_type,
        })
    }
    pub fn set(
        scope: &SyncRef<FunctionVariableScope>,
        pos: ItemPosition,
        components: &Vec<ExpressionAST>,
    ) -> Result<Self, Vec<SemanticError>>
    {
        let components = components.resolve(scope)?;
        let fields: Vec<Field> = components.iter()
            .map(|expr| {
                let field_type = expr.data_type.clone();
                let position = expr.pos;
                Field {
                    attributes: Vec::new(),
                    field_type,
                    position,
                }
            })
            .collect();
        let data_type = DataType::Compound(CompoundDataType::Tuple(fields));
        Ok(Expression {
            body: ExpressionBody::Set(components),
            pos,
            data_type,
        })
    }
    pub fn function_call(
        scope: &SyncRef<FunctionVariableScope>,
        pos: ItemPosition,
        function: &ItemPath,
        arguments: &Vec<ExpressionAST>,
    ) -> Result<Self, Vec<SemanticError>>
    {
        let module = scope.context().module();
        let arguments: Vec<Expression> = arguments.resolve(scope)?;
        let function_item = match module.get_item(function.path.as_path(), &mut Vec::new()) {
            Some(item) => item,
            None => {
                return match function.path.as_path().the_only() {
                    Some(name) => {
                        Expression::std_function_call(scope, pos, name, arguments)
                            .map_err(|e| vec![e])
                    }
                    None => SemanticError::unresolved_item(
                        function.pos,
                        function.path.clone(),
                    )
                        .into_err_vec(),
                };
            }
        };

        let data_type = {
            let function_guard = function_item.read();
            let function = match function_guard.get_function() {
                Some(func) => func,
                None => return SemanticError::expected_item_of_another_type(
                    pos,
                    SemanticItemType::Function,
                    function_guard.get_type(),
                )
                    .into_err_vec(),
            };

            if scope.is_lite_weight() && function.is_lite_weight {
                return SemanticError::not_allowed_here(
                    pos,
                    "not lite-weight functions",
                )
                    .into_err_vec();
            }

            if arguments.len() != function.arguments.len() {
                return SemanticError::wrong_arguments_count(
                    pos,
                    function.arguments.len(),
                    arguments.len(),
                )
                    .into_err_vec();
            }

            for (i, argument) in arguments.iter().enumerate() {
                let (_, target_data_type) = function.arguments.get_index(i)
                    .expect("The argument can not cease to exist immediately after checking the length of the collection");
                if !argument.data_type.can_cast(target_data_type) {
                    return SemanticError::cannot_cast_type(
                        argument.pos,
                        argument.data_type.clone(),
                        target_data_type.clone(),
                    )
                        .into_err_vec();
                }
            }

            function.result.clone()
        };
        Ok(Expression {
            body: ExpressionBody::FunctionCall(function_item, arguments),
            pos,
            data_type,
        })
    }
    pub fn std_function_call(
        scope: &SyncRef<FunctionVariableScope>,
        pos: ItemPosition,
        name: &str,
        arguments: Vec<Expression>,
    ) -> Result<Self, SemanticError> {
        let function = match scope.project().resolve_stdlib_function(name) {
            Some(f) => f,
            None => {
                let mut path = PathBuf::empty();
                path.push(name);
                return Err(SemanticError::unresolved_item(pos, path));
            }
        };

        if scope.is_lite_weight() && function.is_lite_weight {
            return Err(SemanticError::not_allowed_here(
                pos,
                "not lite-weight functions",
            ));
        }

        if !scope.is_aggregate() && function.is_aggregate {
            return Err(SemanticError::not_allowed_here(
                pos,
                "aggregate functions",
            ));
        }

        if arguments.len() != function.arguments.len() {
            return Err(SemanticError::wrong_arguments_count(
                pos,
                function.arguments.len(),
                arguments.len(),
            ));
        }

        for (i, argument) in arguments.iter().enumerate() {
            let target_data_type = function.arguments.get(i)
                .expect("The argument can not cease to exist immediately after checking the length of the collection");
            if !argument.data_type.can_cast(target_data_type) {
                return Err(SemanticError::cannot_cast_type(
                    argument.pos,
                    argument.data_type.clone(),
                    target_data_type.clone(),
                ));
            }
        }

        Ok(Expression {
            body: ExpressionBody::StdFunctionCall(name.to_string(), arguments),
            pos,
            data_type: function.output,
        })
    }
    #[inline]
    pub fn can_be_selected_by_aggregation_query(&self, aggregates: &Vec<Expression>) -> bool {
        aggregates.contains(self)
            || self.body.can_be_selected_by_aggregation_query(aggregates)
    }
}

impl cmp::PartialEq for Expression {
    fn eq(&self, other: &Expression) -> bool {
        (self.data_type == other.data_type)
            &&
            (self.body == other.body)
    }
}
