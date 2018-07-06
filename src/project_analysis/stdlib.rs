use helpers::SyncRef;
use language::{
    BinaryOperator,
    DataType,
    PostfixUnaryOperator,
    PrefixUnaryOperator,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StdLib {
    elements: Vec<StdLibElement>,
}

impl StdLib {
    #[inline]
    pub fn new() -> Self {
        StdLib {
            elements: Vec::new(),
        }
    }

    #[inline]
    pub fn reg_element(&mut self, element: StdLibElement) {
        self.elements.push(element)
    }
    #[inline]
    pub fn reg_postfix_unary_operation(&mut self, operation: StdLibPostfixUnaryOperation) {
        self.reg_element(StdLibElement::PostfixUnaryOperation(Arc::new(operation)))
    }
    #[inline]
    pub fn reg_prefix_unary_operation(&mut self, operation: StdLibPrefixUnaryOperation) {
        self.reg_element(StdLibElement::PrefixUnaryOperation(Arc::new(operation)))
    }
    #[inline]
    pub fn reg_binary_operation(&mut self, operation: StdLibBinaryOperation) {
        self.reg_element(StdLibElement::BinaryOperation(Arc::new(operation)))
    }
    #[inline]
    pub fn reg_function(&mut self, function: StdLibFunction) {
        self.reg_element(StdLibElement::Function(Arc::new(function)))
    }
    pub fn resolve_postfix_unary_operation(&self, operator: PostfixUnaryOperator, input: &DataType) -> Option<&Arc<StdLibPostfixUnaryOperation>> {
        for element in self.elements.iter() {
            if let StdLibElement::PostfixUnaryOperation(op) = element {
                if (op.operator == operator) && input.can_cast(&op.input) {
                    return Some(op);
                }
            }
        }
        None
    }
    pub fn resolve_prefix_unary_operation(&self, operator: PrefixUnaryOperator, input: &DataType) -> Option<&Arc<StdLibPrefixUnaryOperation>> {
        for element in self.elements.iter() {
            if let StdLibElement::PrefixUnaryOperation(op) = element {
                if (op.operator == operator) && input.can_cast(&op.input) {
                    return Some(op);
                }
            }
        }
        None
    }
    pub fn resolve_binary_operation(&self, operator: BinaryOperator, left: &DataType, right: &DataType) -> Option<&Arc<StdLibBinaryOperation>> {
        for element in self.elements.iter() {
            if let StdLibElement::BinaryOperation(op) = element {
                if (op.operator == operator) && left.can_cast(&op.left) && right.can_cast(&op.right) {
                    return Some(op);
                }
            }
        }
        None
    }
    pub fn resolve_function(&self, name: &str) -> Option<&Arc<StdLibFunction>> {
        for element in self.elements.iter() {
            if let StdLibElement::Function(function) = element {
                if function.name == name {
                    return Some(function);
                }
            }
        }
        None
    }
}

impl SyncRef<StdLib> {
    #[inline]
    pub fn resolve_postfix_unary_operation(&self, operator: PostfixUnaryOperator, input: &DataType) -> Option<Arc<StdLibPostfixUnaryOperation>> {
        self.read()
            .resolve_postfix_unary_operation(operator, input)
            .cloned()
    }
    #[inline]
    pub fn resolve_prefix_unary_operation(&self, operator: PrefixUnaryOperator, input: &DataType) -> Option<Arc<StdLibPrefixUnaryOperation>> {
        self.read()
            .resolve_prefix_unary_operation(operator, input)
            .cloned()
    }
    #[inline]
    pub fn resolve_binary_operation(&self, operator: BinaryOperator, left: &DataType, right: &DataType) -> Option<Arc<StdLibBinaryOperation>> {
        self.read()
            .resolve_binary_operation(operator, left, right)
            .cloned()
    }
    #[inline]
    pub fn resolve_function(&self, name: &str) -> Option<Arc<StdLibFunction>> {
        self.read()
            .resolve_function(name)
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StdLibElement {
    PostfixUnaryOperation(Arc<StdLibPostfixUnaryOperation>),
    PrefixUnaryOperation(Arc<StdLibPrefixUnaryOperation>),
    BinaryOperation(Arc<StdLibBinaryOperation>),
    Function(Arc<StdLibFunction>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StdLibPostfixUnaryOperation {
    pub operator: PostfixUnaryOperator,
    pub input: DataType,
    pub output: DataType,
}

impl StdLibPostfixUnaryOperation {
    #[inline]
    pub fn new(operator: PostfixUnaryOperator, input: DataType, output: DataType) -> Self {
        StdLibPostfixUnaryOperation {
            operator,
            input,
            output,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StdLibPrefixUnaryOperation {
    pub operator: PrefixUnaryOperator,
    pub input: DataType,
    pub output: DataType,
}

impl StdLibPrefixUnaryOperation {
    #[inline]
    pub fn new(operator: PrefixUnaryOperator, input: DataType, output: DataType) -> Self {
        StdLibPrefixUnaryOperation {
            operator,
            input,
            output,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StdLibBinaryOperation {
    pub operator: BinaryOperator,
    pub left: DataType,
    pub right: DataType,
    pub output: DataType,
}

impl StdLibBinaryOperation {
    #[inline]
    pub fn new(operator: BinaryOperator, left: DataType, right: DataType, output: DataType) -> Self {
        StdLibBinaryOperation {
            operator,
            left,
            right,
            output,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StdLibFunction {
    pub name: String,
    pub arguments: Vec<DataType>,
    pub output: DataType,
    pub is_aggregate: bool,
    pub is_lite_weight: bool,
}

impl StdLibFunction {
    #[inline]
    pub fn new(name: String) -> Self {
        StdLibFunction {
            name,
            arguments: Vec::new(),
            output: DataType::Void,
            is_aggregate: false,
            is_lite_weight: false,
        }
    }
    #[inline]
    pub fn gets(mut self, arguments: Vec<DataType>) -> Self {
        self.arguments = arguments;
        self
    }
    #[inline]
    pub fn returns(mut self, output: DataType) -> Self {
        self.output = output;
        self
    }
    #[inline]
    pub fn lite_weight(mut self) -> Self {
        self.is_lite_weight = true;
        self
    }
    #[inline]
    pub fn aggregate(mut self) -> Self {
        self.is_aggregate = true;
        self.lite_weight()
    }
}
