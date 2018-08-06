use helpers::{
    parse_index,
    Path,
    PathBuf,
    Resolve,
    SyncRef,
};
use helpers::{
    Assertion,
    is_f32_enough,
};
use language::{
    CompoundDataType,
    DataType,
    Field,
    ItemPath,
    NumberType,
    PrimitiveDataType,
    StringType,
    TSQLFunctionContext,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use project_analysis::{
    FunctionVariable,
    FunctionVariableScope,
    Item,
    SemanticError,
    SemanticItemType,
    StdLibFunction,
};
use std::{
    cmp,
    fmt,
    sync::Arc,
    u8::MAX as U8MAX,
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

impl Literal {
    pub fn fmt(
        &self,
        f: &mut impl fmt::Write,
    ) -> fmt::Result {
        match &self.literal_type {
            LiteralType::NumberLiteral { .. } |
            LiteralType::StringLiteral { .. } |
            LiteralType::BracedExpressionLiteral { .. } =>
                f.write_str(self.text.as_str()),
            LiteralType::KeywordLiteral(keyword) =>
                f.write_str(match keyword {
                    KeywordLiteralType::True => "1",
                    KeywordLiteralType::False => "0",
                    KeywordLiteralType::Null => "null",
                })
        }
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

impl BinaryOperator {
    pub fn get_description(&self) -> &'static str {
        match self {
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
    pub fn get_operator(&self) -> &'static str {
        match self {
            BinaryOperator::Or => "||",
            BinaryOperator::XOr => "<>",
            BinaryOperator::And => "&&",
            BinaryOperator::BitOr => "|",
            BinaryOperator::BitXOr => "^",
            BinaryOperator::BitAnd => "&",
            BinaryOperator::ShiftLeft => "<<",
            BinaryOperator::ShiftRight => ">>",
            BinaryOperator::IsIn => "is in",
            BinaryOperator::Equals => "=",
            BinaryOperator::MoreThanOrEquals => ">=",
            BinaryOperator::MoreThan => ">",
            BinaryOperator::LessThanOrEquals => "<=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::Like => "like",
            BinaryOperator::SoundsLike => "sounds like",
            BinaryOperator::RegExp => "reg exp",
            BinaryOperator::Plus => "+",
            BinaryOperator::Minus => "-",
            BinaryOperator::Times => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Mod => "%",
            BinaryOperator::Div => "div",
            BinaryOperator::Pow => "**",
            BinaryOperator::Interval => "..",
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
        match self {
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
    pub fn get_operator(&self) -> &'static str {
        match self {
            PrefixUnaryOperator::Not => "!",
            PrefixUnaryOperator::All => "all",
            PrefixUnaryOperator::Any => "any",
            PrefixUnaryOperator::Plus => "+",
            PrefixUnaryOperator::Minus => "-",
            PrefixUnaryOperator::Tilde => "~",
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
        match self {
            PostfixUnaryOperator::IsNull => "is null",
            PostfixUnaryOperator::IsTrue => "is true",
            PostfixUnaryOperator::IsFalse => "is false",
            PostfixUnaryOperator::IsUnknown => "is unknown",
        }
    }
    pub fn get_operator(&self) -> &'static str {
        match self {
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
            ExpressionASTBody::Literal(lit_left) => {
                if let ExpressionASTBody::Literal(lit_right) = &other.body {
                    lit_left.assert(lit_right)
                } else { assert_eq!(self.body, other.body) }
            }
            ExpressionASTBody::Reference(ident_left) => {
                if let ExpressionASTBody::Reference(ident_right) = &other.body {
                    assert_eq!(ident_left, ident_right);
                } else { assert_eq!(self.body, other.body) }
            }
            ExpressionASTBody::BinaryOperation(left_left, left_op, left_right) => {
                if let ExpressionASTBody::BinaryOperation(right_left, right_op, right_right) = &other.body {
                    (*left_left).assert(&**right_left);
                    assert_eq!(left_op, right_op);
                    (*left_right).assert(&**right_right);
                } else { assert_eq!(self.body, other.body) }
            }
            ExpressionASTBody::PrefixUnaryOperation(left_op, left) => {
                if let ExpressionASTBody::PrefixUnaryOperation(right_op, right) = &other.body {
                    assert_eq!(left_op, right_op);
                    (*left).assert(&**right);
                } else { assert_eq!(self.body, other.body) }
            }
            ExpressionASTBody::PostfixUnaryOperation(left_op, left) => {
                if let ExpressionASTBody::PostfixUnaryOperation(right_op, right) = &other.body {
                    assert_eq!(left_op, right_op);
                    (*left).assert(&**right);
                } else { assert_eq!(self.body, other.body) }
            }
            ExpressionASTBody::PropertyAccess(left, left_path) => {
                if let ExpressionASTBody::PropertyAccess(right, right_path) = &other.body {
                    assert_eq!(left_path.path, right_path.path);
                    (*left).assert(&**right);
                } else { assert_eq!(self.body, other.body) }
            }
            ExpressionASTBody::Set(left_items) => {
                if let ExpressionASTBody::Set(right_items) = &other.body {
                    left_items.as_slice().assert(&right_items.as_slice());
                } else { assert_eq!(self.body, other.body) }
            }
            ExpressionASTBody::FunctionCall(left_name, left_args) => {
                if let ExpressionASTBody::FunctionCall(right_name, right_args) = &other.body {
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

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for ExpressionAST<'source> {
    type Result = Expression;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        match &self.body {
            ExpressionASTBody::Literal(lit) => {
                Expression::literal(self.pos, lit)
                    .map_err(|e| vec![e])
            }
            ExpressionASTBody::Reference(ident) => {
                Expression::variable(scope, self.pos, ident)
                    .map_err(|e| vec![e])
            }
            ExpressionASTBody::BinaryOperation(left, op, right) => {
                Expression::binary_operation(scope, self.pos, *op, left, right)
            }
            ExpressionASTBody::PostfixUnaryOperation(op, expr) => {
                Expression::postfix_unary_operation(scope, self.pos, *op, expr)
            }
            ExpressionASTBody::PrefixUnaryOperation(op, expr) => {
                Expression::prefix_unary_operation(scope, self.pos, *op, expr)
            }
            ExpressionASTBody::PropertyAccess(expr, path) => {
                Expression::property_access(scope, self.pos, expr, path)
            }
            ExpressionASTBody::Set(components) => {
                Expression::set(scope, self.pos, components)
            }
            ExpressionASTBody::FunctionCall(function, arguments) => {
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
    StdFunctionCall(Arc<StdLibFunction>, Vec<Expression>),
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
        let data_type = var.property_type(pos, Path::empty())?;
        Ok(Expression::variable_access(var, pos, data_type))
    }
    #[inline]
    pub fn variable_access(
        var: SyncRef<FunctionVariable>,
        pos: ItemPosition,
        data_type: DataType,
    ) -> Self {
        Expression {
            body: ExpressionBody::Variable(var),
            pos,
            data_type,
        }
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
            .resolve_binary_operation(pos, op, &left.data_type, &right.data_type)?
            .output
            .clone();
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
            .resolve_postfix_unary_operation(pos, op, &expr.data_type)?
            .output
            .clone();
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
            .resolve_prefix_unary_operation(pos, op, &expr.data_type)?
            .output
            .clone();
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
        let data_type = expr.data_type.property_type(pos, path.path.as_path())?;
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
                Field {
                    attributes: Vec::new(),
                    field_type,
                }
            })
            .collect();
        let data_type = DataType::Compound(CompoundDataType::Tuple(Arc::new(fields)));
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

            if scope.is_lite_weight() && function.result.as_primitive().is_none() {
                return SemanticError::not_allowed_here(
                    pos,
                    "non-primitive function call",
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
                let (_, target) = function.arguments.get_index(i)
                    .expect("The argument can not cease to exist immediately after checking the length of the collection");
                argument.should_cast_to_type(
                    target.read()
                        .data_type()
                        .expect("Function arguments cannot have undefined data type")
                )?;
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

        if scope.is_lite_weight() && !function.is_lite_weight {
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
            argument.should_cast_to_type(target_data_type)?;
        }

        let data_type = function.output.clone();

        Ok(Expression {
            body: ExpressionBody::StdFunctionCall(function, arguments),
            pos,
            data_type,
        })
    }
    #[inline]
    pub fn can_expressions_be_selected_by_aggregation_query<'a, 'b>(
        expressions: impl IntoIterator<Item=&'a Expression>,
        aggregates: impl Clone + IntoIterator<Item=&'b Expression>,
    ) -> Result<bool, Vec<SemanticError>> {
        for expression in expressions {
            if !expression.can_be_selected_by_aggregation_query(aggregates.clone())? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn _is_in_aggregates<'a>(&self, aggregates: impl IntoIterator<Item=&'a Expression>) -> bool {
        for aggregate in aggregates {
            if self == aggregate {
                return true;
            }
        }
        false
    }
    pub fn can_be_selected_by_aggregation_query<'a>(&self, aggregates: impl Clone + IntoIterator<Item=&'a Expression>) -> Result<bool, Vec<SemanticError>> {
        if self._is_in_aggregates(aggregates.clone()) {
            return Ok(true);
        }
        match &self.body {
            ExpressionBody::Literal(_) => Ok(true),
            ExpressionBody::Variable(_) => Ok(false),
            ExpressionBody::BinaryOperation(left, _, right) => {
                Ok(
                    left.can_be_selected_by_aggregation_query(aggregates.clone())?
                        && right.can_be_selected_by_aggregation_query(aggregates)?
                )
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
                Expression::can_expressions_be_selected_by_aggregation_query(expressions, aggregates)
            }
            ExpressionBody::FunctionCall(_, expressions) => {
                Expression::can_expressions_be_selected_by_aggregation_query(expressions, aggregates)
            }
            ExpressionBody::StdFunctionCall(function, expressions) => {
                if function.is_aggregate {
                    let mut errors = Vec::new();
                    for expression in expressions.iter() {
                        if expression.is_aggregate() {
                            errors.push(SemanticError::not_allowed_inside(
                                expression.pos,
                                "aggregate expressions",
                                "aggregate function's argument",
                            ));
                        }
                    }
                    if !errors.is_empty() {
                        return Err(errors);
                    }
                    Ok(true)
                } else {
                    Expression::can_expressions_be_selected_by_aggregation_query(expressions, aggregates)
                }
            }
        }
    }
    #[inline]
    pub fn is_aggregate(&self) -> bool {
        match &self.body {
            ExpressionBody::StdFunctionCall(function, _) => {
                function.is_aggregate
            }
            _ => false,
        }
    }
    pub fn can_be_named(&self) -> Option<String> {
        match &self.body {
            ExpressionBody::Variable(var) => {
                Some(
                    var.read()
                        .name()
                        .to_string()
                )
            }
            ExpressionBody::PropertyAccess(_, path) => {
                path.path.as_path()
                    .pop_right()
                    .map(str::to_string)
            }
            _ => None,
        }
    }
    #[inline]
    pub fn should_cast_to_type(&self, target: &DataType) -> Result<(), SemanticError> {
        self.data_type.should_cast_to(self.pos, target)
    }
    pub fn is_lite_weight(&self) -> bool {
        match &self.body {
            ExpressionBody::Literal(_) => true,
            ExpressionBody::Variable(_) => true,
            ExpressionBody::BinaryOperation(left, _, right) => {
                left.is_lite_weight() && right.is_lite_weight()
            }
            ExpressionBody::PostfixUnaryOperation(_, expr) => {
                expr.is_lite_weight()
            }
            ExpressionBody::PrefixUnaryOperation(_, expr) => {
                expr.is_lite_weight()
            }
            ExpressionBody::PropertyAccess(expr, _) => {
                expr.is_lite_weight()
            }
            ExpressionBody::Set(expressions) => {
                expressions.iter().all(|expr| expr.is_lite_weight())
            }
            ExpressionBody::FunctionCall(function, expressions) => {
                let guard = function.read();
                match guard.get_function() {
                    Some(function) => {
                        function.is_lite_weight
                            && expressions.iter().all(|expr| expr.is_lite_weight())
                    }
                    None => false,
                }
            }
            ExpressionBody::StdFunctionCall(function, expressions) => {
                function.is_lite_weight
                    && expressions.iter().all(|expr| expr.is_lite_weight())
            }
        }
    }
    pub fn get_property(&self, path: Path) -> Option<Expression> {
        if path.is_empty() {
            return Some(self.clone());
        }
        match &self.body {
            ExpressionBody::PropertyAccess(expr, local_path) => {
                let mut deeper_path = local_path.path.clone();
                deeper_path.append(path);
                if let Some(sub_expr) = expr.get_property(deeper_path.as_path()) {
                    return Some(sub_expr);
                }
                if let Ok(data_type) = self.data_type.property_type(self.pos, path) {
                    return Some(Expression {
                        pos: self.pos,
                        data_type,
                        body: ExpressionBody::PropertyAccess(expr.clone(), ItemPath {
                            pos: self.pos,
                            path: deeper_path,
                        }),
                    });
                }
                return None;
            }
            ExpressionBody::Set(expressions) => {
                let index = {
                    let mut path_components = path.components();
                    let first = path_components.next()?;
                    if path_components.next().is_some() { return None; }
                    parse_index(first)?
                };
                if expressions.len() > index {
                    return Some(expressions[index].clone());
                }
            }
            _ => {},
        }
        None
    }
    pub fn get_property_or_wrap(&self, path: Path) -> Option<Expression> {
        if let Some(result) = self.get_property(path.clone()) {
            return Some(result);
        }
        if let Ok(data_type) = self.data_type.property_type(self.pos, path) {
            return Some(Expression {
                pos: self.pos,
                data_type,
                body: ExpressionBody::PropertyAccess(box self.clone(), ItemPath {
                    pos: self.pos,
                    path: path.into(),
                }),
            });
        }
        None
    }
    pub fn fmt_variable(
        f: &mut impl fmt::Write,
        var: &FunctionVariable,
        access: bool,
    ) -> fmt::Result {
        let name = var.name();
        if var.is_automatic() {
            f.write_str(name)?;
            if access {
                f.write_char('.')?;
            }
        } else {
            write!(f, "@{}", name)?;
            if access {
                f.write_char('#')?;
            }
        }
        Ok(())
    }
    pub fn fmt_data_list(
        f: &mut impl fmt::Write,
        data_type: &DataType,
        var_name: &str,
        var_is_automatic: bool,
        last_comma: bool,
    ) -> Result<bool, fmt::Error> {
        if data_type.as_primitive().is_some() {
            if var_is_automatic {
                write!(f, "{}", var_name)?;
            } else {
                write!(f, "@{}", var_name)?;
            }
            return Ok(true);
        }

        let primitives = data_type.primitives(PathBuf::new("#"));

        if primitives.is_empty() {
            return Ok(false);
        }

        let mut primitives = primitives.into_iter().peekable();
        while let Some(primitive) = primitives.next() {
            if var_is_automatic {
                write!(f, "{}.", var_name)?;
            } else {
                write!(f, "@{}#", var_name)?;
            }
            write!(f, "{path} AS {path}", path = primitive.path.data)?;
            if last_comma || primitives.peek().is_some() {
                f.write_str(", ")?;
            }
        }
        Ok(true)
    }
    pub fn fmt_variable_data(
        f: &mut impl fmt::Write,
        var: &FunctionVariable,
    ) -> fmt::Result {
        let data_type = var.data_type()
            .expect("Variables cannot have unknown data-type at generate-time");

        if data_type.as_primitive().is_some() {
            if var.is_automatic() {
                write!(f, "{}", var.name())?;
            } else {
                write!(f, "@{}", var.name())?;
            }
            return Ok(());
        }

        f.write_str("(SELECT ")?;

        if let Some(sub_type) = data_type.as_array() {
            // Если в переменной лежит таблица - нужно выбрать все записи
            if !Expression::fmt_data_list(
                f,
                sub_type,
                "t",
                true,
                false,
            )? {
                f.write_char('0')?;
            }

            if !var.is_automatic() {
                write!(f, " FROM @{} as t", var.name())?;
            }
        } else {
            // Если нет - одну запись той же структуры, что и переменная
            if !Expression::fmt_data_list(
                f,
                data_type,
                var.name(),
                var.is_automatic(),
                false,
            )? {
                f.write_char('0')?;
            }
        }

        f.write_str(")")
    }
    pub fn fmt_property_access(
        f: &mut impl fmt::Write,
        expr: &Expression,
        path: Path,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        let path_buf = path.into_new_buf("#");
        if let ExpressionBody::Variable(var) = &expr.body {
            let var_guard = var.read();
            Expression::fmt_variable(f, &*var_guard, !path_buf.is_empty())?;
            return f.write_str(&path_buf.data)
        }
        if let Some(sub_expr) = expr.get_property(path) {
            return sub_expr.fmt(f, context)
        }
        write!(f, "( SELECT t.{} FROM ", path_buf.data)?;
        expr.fmt(f, context)?;
        f.write_str(" as t )")
    }
    pub fn fmt_function_call(
        f: &mut impl fmt::Write,
        function: &SyncRef<Item>,
        arguments: &[Expression],
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        let function_guard = function.read();
        let function_def = function_guard.get_function()
            .expect("item argument of Statement::fmt_pre_call is not a function!");
        write!(f, "[{}](", function.read().get_path().data)?;
        let mut arguments = arguments.iter()
            .enumerate()
            .peekable();
        while let Some((i, argument)) = arguments.next() {
            let (_, argument_target) = function_def.arguments.get_index(i)
                .expect("Arguments should not have different count at generate-time.");
            let argument_target_guard = argument_target.read();
            let argument_target_data_type = argument_target_guard.data_type()
                .expect("Arguments cannot have unknown data-type at generate-time");

            let mut primitives = argument_target_data_type.primitives(PathBuf::new("#"))
                .into_iter()
                .peekable();

            while let Some(primitive) = primitives.next() {
                match argument.get_property_or_wrap(primitive.path.as_path()) {
                    Some(sub_expr) => sub_expr.fmt(f, context)?,
                    None => argument.fmt(f, context)?,
                }
                if arguments.peek().is_some() || primitives.peek().is_some() {
                    f.write_char(',')?;
                }
            }
        }
        f.write_str(")")
    }
    pub fn fmt(
        &self,
        f: &mut impl fmt::Write,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        match &self.body {
            ExpressionBody::Literal(lit) => lit.fmt(f),
            ExpressionBody::Variable(var) => {
                let var_guard = var.read();
                Expression::fmt_variable_data(f, &*var_guard)
            }
            ExpressionBody::BinaryOperation(left, op, right) => {
                f.write_str("( ")?;
                left.fmt(f, context)?;
                f.write_str(" ")?;
                f.write_str(op.get_operator())?;
                f.write_str(" ")?;
                right.fmt(f, context)?;
                f.write_str(" )")
            }
            ExpressionBody::PostfixUnaryOperation(op, expr) => {
                f.write_str("( ")?;
                f.write_str(op.get_operator())?;
                f.write_str(" ")?;
                expr.fmt(f, context)?;
                f.write_str(" )")
            }
            ExpressionBody::PrefixUnaryOperation(op, expr) => {
                f.write_str("( ")?;
                f.write_str(op.get_operator())?;
                f.write_str(" ")?;
                expr.fmt(f, context)?;
                f.write_str(" )")
            }
            ExpressionBody::PropertyAccess(expr, path) => {
                Expression::fmt_property_access(
                    f,
                    &expr,
                    path.path.as_path(),
                    context,
                )
            }
            ExpressionBody::Set(expressions) => {
                f.write_str("(SELECT ")?;
                if expressions.is_empty() {
                    f.write_str("0")?;
                } else {
                    let max_i = expressions.len() - 1;
                    for (i, expression) in expressions.iter().enumerate() {
                        expression.fmt(f, context)?;
                        write!(f, " as component{}", i)?;
                        if i < max_i {
                            f.write_str(", ")?;
                        }
                    }
                }
                f.write_str(")")
            }
            ExpressionBody::FunctionCall(function, arguments) => {
                let is_primitive = {
                    let function_guard = function.read();
                    if let Some(function_def) = function_guard.get_function() {
                        function_def.result.as_primitive()
                            .is_some()
                            && function_def.is_lite_weight
                    } else {
                        false
                    }
                };
                if is_primitive {
                    Expression::fmt_function_call(
                        f,
                        function,
                        &arguments,
                        context,
                    )
                } else {
                    let var = context.add_pre_calc_call(function, &arguments)?;
                    let var_guard = var.read();
                    Expression::fmt_variable_data(f, &*var_guard)
                }
            }
            ExpressionBody::StdFunctionCall(function, arguments) => {
                write!(f, "{}(", function.name)?;
                let mut arguments = arguments.iter().peekable();
                while let Some(argument) = arguments.next() {
                    argument.fmt(f, context)?;
                    if arguments.peek().is_some() {
                        f.write_str(", ")?;
                    }
                }
                f.write_str(")")
            }
        }
    }
}

impl cmp::PartialEq for Expression {
    fn eq(&self, other: &Expression) -> bool {
        (self.data_type == other.data_type)
            &&
            (self.body == other.body)
    }
}
