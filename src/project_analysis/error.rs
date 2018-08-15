use helpers::{
    decimal_unsigned_length,
    IntoStatic,
    PathBuf,
    write_line_numbers_columns_row,
    write_pointer_line,
};
use language::{
    BinaryOperator,
    DataType,
    PostfixUnaryOperator,
    PrefixUnaryOperator,
};
use lexeme_scanner::{
    ItemPosition,
    ScannerError,
    ScannerErrorKind,
};
use parser_basics::{
    ParserErrorItem,
    ParserErrorKind,
};
use project_analysis::{
    SemanticItemType,
    Text,
};
use std::{
    cmp::max,
    error::Error,
    fmt,
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticErrorKind {
    Empty,
    UnresolvedItem {
        path: PathBuf,
    },
    SuperOfRoot,
    ItemNameNotSpecified,
    DuplicateDefinition {
        name: String,
        item_type: SemanticItemType,
    },
    ScannerError {
        kind: ScannerErrorKind,
    },
    ParserError {
        kind: ParserErrorKind<'static>,
    },
    ExpectedItemOfAnotherType {
        expected: SemanticItemType,
        got: SemanticItemType,
    },
    EmptyPrimaryKey,
    NotInScope {
        name: String,
    },
    WrongProperty {
        property: String,
    },
    VariableTypeIsUnknown {
        name: String,
    },
    NotSupportedYet {
        feature: &'static str,
    },
    WrongArgumentsCount {
        expected: usize,
        got: usize,
    },
    CannotCastType {
        source: DataType,
        target: DataType,
    },
    BinaryOperationCannotBePerformed {
        operator: BinaryOperator,
        left: DataType,
        right: DataType,
    },
    PostfixUnaryOperationCannotBePerformed {
        operator: PostfixUnaryOperator,
        input: DataType,
    },
    PrefixUnaryOperationCannotBePerformed {
        operator: PrefixUnaryOperator,
        input: DataType,
    },
    NotAllowedHere {
        feature: &'static str,
    },
    NotAllowedInside {
        feature: &'static str,
        output_feature: &'static str,
    },
    ExpectedExpressionOfAnotherType {
        expected: DataType,
        got: DataType,
    },
    CannotModifyReadOnlyVariable {
        name: String,
    },
    UnreachableStatement,
    NotAllBranchesReturns,
    CannotDoWithDataSource {
        action: &'static str,
    },
    ValueListWithWrongLength {
        expected: usize,
        got: usize,
    },
    SelectWithWrongColumnCount {
        expected: usize,
        got: usize,
    },
}

impl Default for SemanticErrorKind {
    fn default() -> Self {
        SemanticErrorKind::Empty
    }
}

impl fmt::Display for SemanticErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SemanticErrorKind::Empty => write!(f, "empty error"),
            SemanticErrorKind::UnresolvedItem { path } => write!(f, "unresolved item {}", path.data),
            SemanticErrorKind::SuperOfRoot => write!(f, "cannot get 'super' of root module"),
            SemanticErrorKind::ItemNameNotSpecified => write!(f, "name of using item should be specified"),
            SemanticErrorKind::DuplicateDefinition { name, item_type } => write!(f, "there is already declared {} name {}", item_type, name),
            SemanticErrorKind::ScannerError { kind } => write!(f, "{}", kind),
            SemanticErrorKind::ParserError { kind } => write!(f, "{}", kind),
            SemanticErrorKind::ExpectedItemOfAnotherType { expected, got } => write!(f, "{} expected here, got {}", expected, got),
            SemanticErrorKind::EmptyPrimaryKey => write!(f, "empty primary key"),
            SemanticErrorKind::NotInScope { name } => write!(f, "{} is not in the scope", name),
            SemanticErrorKind::WrongProperty { property } => write!(f, "property {} is not in the scope", property),
            SemanticErrorKind::VariableTypeIsUnknown { name } => write!(f, "type of variable {} is unknown", name),
            SemanticErrorKind::NotSupportedYet { feature } => write!(f, "{} is not supported yet", feature),
            SemanticErrorKind::WrongArgumentsCount { expected, got } => write!(f, "expected {} arguments, got {}", expected, got),
            SemanticErrorKind::CannotCastType { source, target } => write!(f, "cannot cast type {} to {}", source, target),
            SemanticErrorKind::BinaryOperationCannotBePerformed { operator, left, right } => write!(f, "operation \"{}\" cannot be performed on {} and {}", operator, left, right),
            SemanticErrorKind::PostfixUnaryOperationCannotBePerformed { operator, input } => write!(f, "operation \"{}\" cannot be performed on {}", operator, input),
            SemanticErrorKind::PrefixUnaryOperationCannotBePerformed { operator, input } => write!(f, "operation \"{}\" cannot be performed on {}", operator, input),
            SemanticErrorKind::NotAllowedHere { feature } => write!(f, "{} is not allowed here", feature),
            SemanticErrorKind::NotAllowedInside { feature, output_feature } => write!(f, "{} is not allowed inside {}", feature, output_feature),
            SemanticErrorKind::ExpectedExpressionOfAnotherType { expected, got } => write!(f, "expected expression of type {}, got {}", expected, got),
            SemanticErrorKind::CannotModifyReadOnlyVariable { name } => write!(f, "can't modify read-only variable {}", name),
            SemanticErrorKind::UnreachableStatement => write!(f, "unreachable statement"),
            SemanticErrorKind::NotAllBranchesReturns => write!(f, "not all branches of code return a value"),
            SemanticErrorKind::CannotDoWithDataSource { action } => write!(f, "can't {} this data-source", action),
            SemanticErrorKind::ValueListWithWrongLength { expected, got } => write!(f, "expected value list of {} elements, got {}", expected, got),
            SemanticErrorKind::SelectWithWrongColumnCount { expected, got } => write!(f, "expected selection with {} columns, got with {}", expected, got),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SemanticError {
    pub pos: ItemPosition,
    pub kind: SemanticErrorKind,
    pub text: Option<Arc<Text>>,
}

impl SemanticError {
    #[inline]
    pub fn new(pos: ItemPosition, kind: SemanticErrorKind) -> Self {
        SemanticError { pos, kind, text: None }
    }
    #[inline]
    pub fn empty(pos: ItemPosition) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::Empty, text: None }
    }
    #[inline]
    pub fn unresolved_item(pos: ItemPosition, path: PathBuf) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::UnresolvedItem { path }, text: None }
    }
    #[inline]
    pub fn super_of_root(pos: ItemPosition) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::SuperOfRoot, text: None }
    }
    #[inline]
    pub fn item_name_not_specified(pos: ItemPosition) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::ItemNameNotSpecified, text: None }
    }
    #[inline]
    pub fn duplicate_definition(pos: ItemPosition, name: String, item_type: SemanticItemType) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::DuplicateDefinition { name, item_type }, text: None }
    }
    #[inline]
    pub fn scanner_error(error: ScannerError) -> Self {
        let ScannerError { kind, pos } = error;
        SemanticError {
            pos: pos.into_item_pos(" "),
            kind: SemanticErrorKind::ScannerError { kind },
            text: None,
        }
    }
    #[inline]
    pub fn parser_error(error: ParserErrorItem) -> Self {
        let ParserErrorItem { kind, pos } = error;
        SemanticError {
            pos: pos.map_or_else(
                Default::default,
                |pos| pos.into_item_pos(" "),
            ),
            kind: SemanticErrorKind::ParserError { kind: kind.into_static() },
            text: None,
        }
    }
    #[inline]
    pub fn expected_item_of_another_type(pos: ItemPosition, expected: SemanticItemType, got: SemanticItemType) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::ExpectedItemOfAnotherType { expected, got }, text: None }
    }
    #[inline]
    pub fn empty_primary_key(pos: ItemPosition) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::EmptyPrimaryKey, text: None }
    }
    #[inline]
    pub fn not_in_scope(pos: ItemPosition, name: String) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::NotInScope { name }, text: None }
    }
    #[inline]
    pub fn wrong_property(pos: ItemPosition, property: String) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::WrongProperty { property }, text: None }
    }
    #[inline]
    pub fn variable_type_is_unknown(pos: ItemPosition, name: String) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::VariableTypeIsUnknown { name }, text: None }
    }
    #[inline]
    pub fn not_supported_yet(pos: ItemPosition, feature: &'static str) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::NotSupportedYet { feature }, text: None }
    }
    #[inline]
    pub fn wrong_arguments_count(pos: ItemPosition, expected: usize, got: usize) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::WrongArgumentsCount { expected, got }, text: None }
    }
    #[inline]
    pub fn cannot_cast_type(pos: ItemPosition, source: DataType, target: DataType) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::CannotCastType { source, target }, text: None }
    }
    #[inline]
    pub fn binary_operation_cannot_be_performed(pos: ItemPosition, operator: BinaryOperator, left: DataType, right: DataType) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::BinaryOperationCannotBePerformed { operator, left, right }, text: None }
    }
    #[inline]
    pub fn postfix_unary_operation_cannot_be_performed(pos: ItemPosition, operator: PostfixUnaryOperator, input: DataType) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::PostfixUnaryOperationCannotBePerformed { operator, input }, text: None }
    }
    #[inline]
    pub fn prefix_unary_operation_cannot_be_performed(pos: ItemPosition, operator: PrefixUnaryOperator, input: DataType) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::PrefixUnaryOperationCannotBePerformed { operator, input }, text: None }
    }
    #[inline]
    pub fn not_allowed_here(pos: ItemPosition, feature: &'static str) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::NotAllowedHere { feature }, text: None }
    }
    #[inline]
    pub fn not_allowed_inside(pos: ItemPosition, feature: &'static str, output_feature: &'static str) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::NotAllowedInside { feature, output_feature }, text: None }
    }
    #[inline]
    pub fn expected_expression_of_another_type(pos: ItemPosition, expected: DataType, got: DataType) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::ExpectedExpressionOfAnotherType { expected, got }, text: None }
    }
    #[inline]
    pub fn cannot_modify_readonly_variable(pos: ItemPosition, name: String) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::CannotModifyReadOnlyVariable { name }, text: None }
    }
    #[inline]
    pub fn unreachable_statement(pos: ItemPosition) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::UnreachableStatement, text: None }
    }
    #[inline]
    pub fn not_all_branches_returns(pos: ItemPosition) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::NotAllBranchesReturns, text: None }
    }
    #[inline]
    pub fn cannot_do_with_datasource(pos: ItemPosition, action: &'static str) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::CannotDoWithDataSource { action }, text: None }
    }
    #[inline]
    pub fn value_list_with_wrong_length(pos: ItemPosition, expected: usize, got: usize) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::ValueListWithWrongLength { expected, got }, text: None }
    }
    #[inline]
    pub fn select_with_wrong_column_count(pos: ItemPosition, expected: usize, got: usize) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::SelectWithWrongColumnCount { expected, got }, text: None }
    }
    #[inline]
    pub fn set_text(&mut self, text: Arc<Text>) {
        self.text = Some(text);
    }
    #[inline]
    pub fn into_err_vec<T>(self) -> Result<T, Vec<Self>> {
        Err(vec![self])
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.text {
            Some(arc) => writeln!(f, "  in {} on {}", &arc.name, self.pos.begin)?,
            None => writeln!(f, "  on {}", self.pos.begin)?,
        }
        writeln!(f, "  error: {}", self.kind)?;
        let text = match &self.text {
            Some(arc) => arc,
            None => return writeln!(f, "   | text is unspecified."),
        };
        let lines = match self.pos.lines() {
            0 => return writeln!(f, "   | text is unspecified."),
            line_count => text.text.lines()
                .skip(self.pos.begin.line - 1)
                .take(line_count)
                .enumerate(),
        };
        let mut no_lines = true;
        let max_line_num_length = max(3, decimal_unsigned_length(self.pos.end.line));
        for (i, line) in lines {
            if no_lines {
                no_lines = false;
                write_line_numbers_columns_row(f, max_line_num_length, None)?;
                writeln!(f, "")?;
            }
            let line_number = self.pos.begin.line + i;
            write_line_numbers_columns_row(f, max_line_num_length, Some(line_number))?;
            writeln!(f, "{}", line)?;
            if i == 0 {
                if self.pos.begin.line != self.pos.end.line {
                    write_pointer_line(f, line, max_line_num_length, self.pos.begin.column, line.len())?;
                } else {
                    write_pointer_line(f, line, max_line_num_length, self.pos.begin.column, self.pos.end.column)?;
                }
            } else {
                if line_number == self.pos.end.line {
                    write_pointer_line(f, line, max_line_num_length, 0, self.pos.end.column)?;
                } else {
                    write_pointer_line(f, line, max_line_num_length, 0, line.len())?;
                }
            }
        }
        if no_lines {
            writeln!(f, "   | text is unspecified.")
        } else {
            writeln!(f, "")
        }
    }
}

impl From<SemanticError> for Vec<SemanticError> {
    #[inline]
    fn from(e: SemanticError) -> Self {
        vec![e]
    }
}

impl Error for SemanticError {
    #[inline]
    fn description(&self) -> &str {
        "Semantic error"
    }

    #[inline]
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SemanticErrors {
    pub errors: Vec<SemanticError>,
}

impl fmt::Display for SemanticErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Found some errors:")?;
        for error in &self.errors {
            writeln!(f, "{}", error)?;
        }
        Ok(())
    }
}

impl Error for SemanticErrors {
    #[inline]
    fn description(&self) -> &str {
        "Semantic errors"
    }

    #[inline]
    fn cause(&self) -> Option<&dyn Error> {
        self.errors.first()
            .map(|err| err as &dyn Error)
    }
}

impl From<SemanticError> for SemanticErrors {
    #[inline]
    fn from(e: SemanticError) -> Self {
        From::<Vec<SemanticError>>::from(vec![e])
    }
}

impl From<Vec<SemanticError>> for SemanticErrors {
    #[inline]
    fn from(errors: Vec<SemanticError>) -> Self {
        Self {
            errors,
        }
    }
}
