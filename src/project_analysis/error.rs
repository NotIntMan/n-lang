use std::mem::replace;
use helpers::group::Appendable;
use lexeme_scanner::ItemPosition;
use parser_basics::StaticIdentifier;
use super::context::SemanticItemType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticError {
    Empty,
    UnresolvedDependency {
        path: Vec<StaticIdentifier>,
    },
    DuplicateDefinition {
        name: StaticIdentifier,
        pos: ItemPosition,
        item_type: SemanticItemType,
    },
}

impl Default for SemanticError {
    fn default() -> Self {
        SemanticError::Empty
    }
}

impl Appendable for SemanticError {
    fn append(&mut self, other: Self) -> Option<Self> {
        if (*self == other) || (other == SemanticError::Empty) {
            return None;
        }
        if *self == SemanticError::Empty {
            replace(self, other);
            return None;
        }
        Some(other)
    }
}
