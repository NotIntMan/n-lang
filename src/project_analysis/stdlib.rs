use helpers::SyncRef;
use language::{
    BinaryOperator,
    DataType,
    PostfixUnaryOperator,
    PrefixUnaryOperator,
};

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
        self.reg_element(StdLibElement::PostfixUnaryOperation(operation))
    }
    #[inline]
    pub fn reg_prefix_unary_operation(&mut self, operation: StdLibPrefixUnaryOperation) {
        self.reg_element(StdLibElement::PrefixUnaryOperation(operation))
    }
    #[inline]
    pub fn reg_binary_operation(&mut self, operation: StdLibBinaryOperation) {
        self.reg_element(StdLibElement::BinaryOperation(operation))
    }
    #[inline]
    pub fn reg_function(&mut self, function: StdLibFunction) {
        self.reg_element(StdLibElement::Function(function))
    }
    pub fn resolve_postfix_unary_operation(&self, operator: PostfixUnaryOperator, input: &DataType) -> Option<&StdLibPostfixUnaryOperation> {
        for element in self.elements.iter() {
            if let &StdLibElement::PostfixUnaryOperation(ref op) = element {
                if (op.operator == operator) && input.can_cast(&op.input) {
                    return Some(op);
                }
            }
        }
        None
    }
    pub fn resolve_prefix_unary_operation(&self, operator: PrefixUnaryOperator, input: &DataType) -> Option<&StdLibPrefixUnaryOperation> {
        for element in self.elements.iter() {
            if let &StdLibElement::PrefixUnaryOperation(ref op) = element {
                if (op.operator == operator) && input.can_cast(&op.input) {
                    return Some(op);
                }
            }
        }
        None
    }
    pub fn resolve_binary_operation(&self, operator: BinaryOperator, left: &DataType, right: &DataType) -> Option<&StdLibBinaryOperation> {
        for element in self.elements.iter() {
            if let &StdLibElement::BinaryOperation(ref op) = element {
                if (op.operator == operator) && left.can_cast(&op.left) && right.can_cast(&op.right) {
                    return Some(op);
                }
            }
        }
        None
    }
    pub fn resolve_function(&self, name: &str) -> Option<&StdLibFunction> {
        for element in self.elements.iter() {
            if let &StdLibElement::Function(ref function) = element {
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
    pub fn resolve_postfix_unary_operation(&self, operator: PostfixUnaryOperator, input: &DataType) -> Option<StdLibPostfixUnaryOperation> {
        self.read()
            .resolve_postfix_unary_operation(operator, input)
            .cloned()
    }
    #[inline]
    pub fn resolve_prefix_unary_operation(&self, operator: PrefixUnaryOperator, input: &DataType) -> Option<StdLibPrefixUnaryOperation> {
        self.read()
            .resolve_prefix_unary_operation(operator, input)
            .cloned()
    }
    #[inline]
    pub fn resolve_binary_operation(&self, operator: BinaryOperator, left: &DataType, right: &DataType) -> Option<StdLibBinaryOperation> {
        self.read()
            .resolve_binary_operation(operator, left, right)
            .cloned()
    }
    #[inline]
    pub fn resolve_function(&self, name: &str) -> Option<StdLibFunction> {
        self.read()
            .resolve_function(name)
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StdLibElement {
    PostfixUnaryOperation(StdLibPostfixUnaryOperation),
    PrefixUnaryOperation(StdLibPrefixUnaryOperation),
    BinaryOperation(StdLibBinaryOperation),
    Function(StdLibFunction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StdLibPostfixUnaryOperation {
    pub operator: PostfixUnaryOperator,
    pub input: DataType,
    pub output: DataType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StdLibPrefixUnaryOperation {
    pub operator: PrefixUnaryOperator,
    pub input: DataType,
    pub output: DataType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StdLibBinaryOperation {
    pub operator: BinaryOperator,
    pub left: DataType,
    pub right: DataType,
    pub output: DataType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StdLibFunction {
    pub name: String,
    pub arguments: Vec<DataType>,
    pub output: DataType,
    pub is_aggregate: bool,
    pub has_side_effects: bool,
}

impl StdLibFunction {
    #[inline]
    pub fn new(name: String) -> Self {
        StdLibFunction {
            name,
            arguments: Vec::new(),
            output: DataType::Void,
            is_aggregate: false,
            has_side_effects: true,
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
    pub fn without_side_effects(mut self) -> Self {
        self.has_side_effects = false;
        self
    }
    #[inline]
    pub fn aggregate(mut self) -> Self {
        self.is_aggregate = true;
        self.without_side_effects()
    }
}
