use std::mem::replace;
use helpers::group::Appendable;
use helpers::into_static::IntoStatic;
use lexeme_scanner::{
    ItemPosition,
    ScannerError,
    ScannerErrorKind,
};
use parser_basics::{
    StaticIdentifier,
    ParserErrorItem,
    ParserErrorKind,
};
use super::context::SemanticItemType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticErrorKind {
    Empty,
    UnresolvedDependency {
        path: Vec<StaticIdentifier>,
    },
    SuperOfGlobal,
    ItemNameNotSpecified,
    DuplicateDefinition {
        name: StaticIdentifier,
        item_type: SemanticItemType,
    },
    ScannerError {
        kind: ScannerErrorKind,
    },
    ParserError {
        kind: ParserErrorKind<'static>,
    },
//    SemanticPanic {
//        message: String,
//    },
}

impl Default for SemanticErrorKind {
    fn default() -> Self {
        SemanticErrorKind::Empty
    }
}

impl Appendable for SemanticErrorKind {
    fn append(&mut self, other: Self) -> Option<Self> {
        if (*self == other) || (other == SemanticErrorKind::Empty) {
            return None;
        }
        if *self == SemanticErrorKind::Empty {
            replace(self, other);
            return None;
        }
        Some(other)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SemanticError {
    pub pos: ItemPosition,
    pub kind: SemanticErrorKind,
}

impl SemanticError {
    #[inline]
    pub fn new(pos: ItemPosition, kind: SemanticErrorKind) -> Self {
        SemanticError { pos, kind }
    }
    #[inline]
    pub fn empty(pos: ItemPosition) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::Empty }
    }
    #[inline]
    pub fn unresolved_dependency(pos: ItemPosition, path: Vec<StaticIdentifier>) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::UnresolvedDependency { path } }
    }
    #[inline]
    pub fn super_of_global(pos: ItemPosition) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::SuperOfGlobal }
    }
    #[inline]
    pub fn item_name_not_specified(pos: ItemPosition) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::ItemNameNotSpecified }
    }
    #[inline]
    pub fn duplicate_definition(pos: ItemPosition, name: StaticIdentifier, item_type: SemanticItemType) -> Self {
        SemanticError { pos, kind: SemanticErrorKind::DuplicateDefinition { name, item_type } }
    }
    #[inline]
    pub fn scanner_error(error: ScannerError) -> Self {
        let ScannerError { kind, pos } = error;
        SemanticError {
            pos: pos.into_item_pos(" "),
            kind: SemanticErrorKind::ScannerError { kind },
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
        }
    }
//    #[inline]
//    pub fn semantic_panic<S: ToString>(message: S) -> Self {
//        SemanticError {
//            pos: ItemPosition::default(),
//            kind: SemanticErrorKind::SemanticPanic {
//                message: message.to_string(),
//            }
//        }
//    }
}

impl Appendable for SemanticError {
    fn append(&mut self, other: Self) -> Option<Self> {
        let SemanticError { pos, kind } = other;
        if self.pos != pos {
            return Some(SemanticError { pos, kind });
        }
        self.kind.append(kind)
            .map(move |kind| SemanticError { pos, kind })
    }
}
