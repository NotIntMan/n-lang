use n_lang::{
    project_analysis::{
        StdLib,
        StdLibBinaryOperation,
        StdLibFunction,
        StdLibPrefixUnaryOperation,
    },
    language::{
        BinaryOperator,
        DataType,
        NumberType,
        PrefixUnaryOperator,
        PrimitiveDataType,
    },
};

const BOOLEAN_TYPE: DataType = DataType::Primitive(PrimitiveDataType::Number(NumberType::Boolean));
const INT_TYPE: DataType = DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
    size: 32,
    unsigned: false,
    zerofill: false,
}));

#[inline]
fn reg_boolean(
    target: &mut StdLib,
    bool_type: DataType,
) {
    target.reg_prefix_unary_operation(StdLibPrefixUnaryOperation {
        operator: PrefixUnaryOperator::Not,
        input: bool_type.clone(),
        output: bool_type.clone(),
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Or,
        left: bool_type.clone(),
        right: bool_type.clone(),
        output: bool_type.clone(),
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::And,
        left: bool_type.clone(),
        right: bool_type.clone(),
        output: bool_type.clone(),
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Equals,
        left: bool_type.clone(),
        right: bool_type.clone(),
        output: BOOLEAN_TYPE,
    });
}

#[inline]
fn reg_arithmetic(
    target: &mut StdLib,
    number_type: DataType,
) {
    target.reg_prefix_unary_operation(StdLibPrefixUnaryOperation {
        operator: PrefixUnaryOperator::Plus,
        input: number_type.clone(),
        output: number_type.clone(),
    });

    target.reg_prefix_unary_operation(StdLibPrefixUnaryOperation {
        operator: PrefixUnaryOperator::Minus,
        input: number_type.clone(),
        output: number_type.clone(),
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Equals,
        left: number_type.clone(),
        right: number_type.clone(),
        output: BOOLEAN_TYPE,
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::MoreThanOrEquals,
        left: number_type.clone(),
        right: number_type.clone(),
        output: BOOLEAN_TYPE,
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::MoreThan,
        left: number_type.clone(),
        right: number_type.clone(),
        output: BOOLEAN_TYPE,
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::LessThanOrEquals,
        left: number_type.clone(),
        right: number_type.clone(),
        output: BOOLEAN_TYPE,
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::LessThan,
        left: number_type.clone(),
        right: number_type.clone(),
        output: BOOLEAN_TYPE,
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Plus,
        left: number_type.clone(),
        right: number_type.clone(),
        output: number_type.clone(),
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Minus,
        left: number_type.clone(),
        right: number_type.clone(),
        output: number_type.clone(),
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Times,
        left: number_type.clone(),
        right: number_type.clone(),
        output: number_type.clone(),
    });

    target.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Divide,
        left: number_type.clone(),
        right: number_type.clone(),
        output: number_type.clone(),
    });

    target.reg_function(
        StdLibFunction::new("max".to_string())
            .gets(vec![number_type.clone()])
            .returns(number_type.clone())
            .aggregate()
    );

    target.reg_function(
        StdLibFunction::new("min".to_string())
            .gets(vec![number_type.clone()])
            .returns(number_type.clone())
            .aggregate()
    );

    target.reg_function(
        StdLibFunction::new("sum".to_string())
            .gets(vec![number_type.clone()])
            .returns(number_type.clone())
            .aggregate()
    );

    target.reg_function(
        StdLibFunction::new("avg".to_string())
            .gets(vec![number_type.clone()])
            .returns(number_type.clone())
            .aggregate()
    );

    target.reg_function(
        StdLibFunction::new("count".to_string())
            .gets(vec![number_type.clone()])
            .returns(INT_TYPE)
            .aggregate()
    );
}

#[inline]
fn reg_int_arithmetic(
    target: &mut StdLib,
    size: u8,
) {
    reg_arithmetic(target, DataType::Primitive(PrimitiveDataType::Number(
        NumberType::Integer {
            size,
            zerofill: true,
            unsigned: true,
        }
    )));

    reg_arithmetic(target, DataType::Primitive(PrimitiveDataType::Number(
        NumberType::Integer {
            size,
            zerofill: true,
            unsigned: false,
        }
    )));

    reg_arithmetic(target, DataType::Primitive(PrimitiveDataType::Number(
        NumberType::Integer {
            size,
            zerofill: false,
            unsigned: true,
        }
    )));

    reg_arithmetic(target, DataType::Primitive(PrimitiveDataType::Number(
        NumberType::Integer {
            size,
            zerofill: false,
            unsigned: false,
        }
    )));
}

#[inline]
fn reg_float_arithmetic(target: &mut StdLib) {
    reg_arithmetic(target, DataType::Primitive(PrimitiveDataType::Number(NumberType::Float {
        double: false,
        size: None,
    })));
    reg_arithmetic(target, DataType::Primitive(PrimitiveDataType::Number(NumberType::Float {
        double: true,
        size: None,
    })));
}

pub fn build_ms_sql_std_lib() -> StdLib {
    let mut stdlib = StdLib::new();
    reg_boolean(&mut stdlib, BOOLEAN_TYPE);
    reg_int_arithmetic(&mut stdlib, 8);
    reg_int_arithmetic(&mut stdlib, 16);
    reg_int_arithmetic(&mut stdlib, 32);
    reg_int_arithmetic(&mut stdlib, 64);
    reg_float_arithmetic(&mut stdlib);
    stdlib
}
