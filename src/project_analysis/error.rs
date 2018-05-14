//use std::mem::replace;
use std::fmt;
use std::cmp::max;
use std::sync::Arc;
//use helpers::Appendable;
use helpers::{
    decimal_unsigned_length,
    IntoStatic,
    PathBuf,
    write_pointer_line,
    write_line_numbers_columns_row,
};
use lexeme_scanner::{
    ItemPosition,
    ScannerError,
    ScannerErrorKind,
};
use parser_basics::{
//    StaticIdentifier,
ParserErrorItem,
ParserErrorKind,
};
//use language::others::write_path;
use language::{
    DataType,
    BinaryOperator,
    PostfixUnaryOperator,
    PrefixUnaryOperator,
};
use project_analysis::{
    SemanticItemType,
    Text,
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
}

impl Default for SemanticErrorKind {
    fn default() -> Self {
        SemanticErrorKind::Empty
    }
}

//impl Appendable for SemanticErrorKind {
//    fn append(&mut self, other: Self) -> Option<Self> {
//        if (*self == other) || (other == SemanticErrorKind::Empty) {
//            return None;
//        }
//        if *self == SemanticErrorKind::Empty {
//            replace(self, other);
//            return None;
//        }
//        Some(other)
//    }
//}

impl fmt::Display for SemanticErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &SemanticErrorKind::Empty => write!(f, "empty error"),
            &SemanticErrorKind::UnresolvedItem { ref path } => write!(f, "unresolved item {}", path.data),
            &SemanticErrorKind::SuperOfRoot => write!(f, "cannot get 'super' of root module"),
            &SemanticErrorKind::ItemNameNotSpecified => write!(f, "name of using item should be specified"),
            &SemanticErrorKind::DuplicateDefinition { ref name, item_type } => write!(f, "there is already declared {} name {}", item_type, name),
            &SemanticErrorKind::ScannerError { ref kind } => write!(f, "{}", kind),
            &SemanticErrorKind::ParserError { ref kind } => write!(f, "{}", kind),
            &SemanticErrorKind::ExpectedItemOfAnotherType { ref expected, ref got } => write!(f, "{} expected here, got {}", expected, got),
            &SemanticErrorKind::EmptyPrimaryKey => write!(f, "empty primary key"),
            &SemanticErrorKind::NotInScope { ref name } => write!(f, "{} is not in the scope", name),
            &SemanticErrorKind::WrongProperty { ref property } => write!(f, "property {} is not in the scope", property),
            &SemanticErrorKind::VariableTypeIsUnknown { ref name } => write!(f, "type of variable {} is unknown", name),
            &SemanticErrorKind::NotSupportedYet { ref feature } => write!(f, "{} is not supported yet", feature),
            &SemanticErrorKind::WrongArgumentsCount { ref expected, ref got } => write!(f, "expected {} arguments, got {}", expected, got),
            &SemanticErrorKind::CannotCastType { ref source, ref target } => write!(f, "cannot cast type {} to {}", source, target),
            &SemanticErrorKind::BinaryOperationCannotBePerformed { ref operator, ref left, ref right } => write!(f, "operation \"{}\" cannot be performed on {} and {}", operator, left, right),
            &SemanticErrorKind::PostfixUnaryOperationCannotBePerformed { ref operator, ref input } => write!(f, "operation \"{}\" cannot be performed on {}", operator, input),
            &SemanticErrorKind::PrefixUnaryOperationCannotBePerformed { ref operator, ref input } => write!(f, "operation \"{}\" cannot be performed on {}", operator, input),
            &SemanticErrorKind::NotAllowedHere { ref feature } => write!(f, "{} is not allowed here", feature),
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
            &Some(ref arc) => writeln!(f, "  in {} on {}", &arc.name, self.pos.begin)?,
            &None => writeln!(f, "  on {}", self.pos.begin)?,
        }
        writeln!(f, "  error: {}", self.kind)?;
        let text = match &self.text {
            &Some(ref arc) => arc,
            &None => return writeln!(f, "   | text is unspecified."),
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

//impl Appendable for SemanticError {
//    fn append(&mut self, other: Self) -> Option<Self> {
//        let SemanticError { pos, kind, text } = other;
//        if self.pos != pos
//            ||
//            self.text != text
//            {
//                return Some(SemanticError { pos, kind, text });
//            }
//        self.kind.append(kind)
//            .map(move |kind| SemanticError { pos, kind, text })
//    }
//}
