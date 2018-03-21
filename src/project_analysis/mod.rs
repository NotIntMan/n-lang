use std::hash::{
    Hash,
    BuildHasher,
};
use std::mem::replace;
use indexmap::IndexMap;
use helpers::group::Appendable;
use lexeme_scanner::ItemPosition;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Project;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyReference {
    pub dependency_id: usize,
    pub type_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticItemType {
    Field,
    DataType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticContext<'source>(&'source str);

impl<'source> SemanticContext<'source> {
    pub fn resolve(&mut self, _item_type: SemanticItemType, path: &[&'source str]) -> Result<DependencyReference, SemanticError<'source>> {
        Err(SemanticError::UnresolvedDependency { path: path.iter().map(|r| *r).collect() })
    }
    pub fn error(&mut self, _error: SemanticError<'source>) {}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticError<'source> {
    Empty,
    UnresolvedDependency {
        path: Vec<&'source str>,
    },
    DuplicateDefinition {
        name: &'source str,
        pos: ItemPosition,
        item_type: SemanticItemType,
    },
}

impl<'source> Default for SemanticError<'source> {
    fn default() -> Self {
        SemanticError::Empty
    }
}

impl<'source> Appendable for SemanticError<'source> {
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

pub trait SemanticResolve {
    fn is_resolved(&self) -> bool;
    fn try_resolve(&mut self, context: &mut SemanticContext);
}

impl<T: SemanticResolve> SemanticResolve for [T] {
    fn is_resolved(&self) -> bool {
        self.iter()
            .all(|item| (*item).is_resolved())
    }
    fn try_resolve(&mut self, context: &mut SemanticContext) {
        for item in self.iter_mut() {
            item.try_resolve(context);
        }
    }
}

impl<T: SemanticResolve> SemanticResolve for Vec<T> {
    fn is_resolved(&self) -> bool { self.as_slice().is_resolved() }
    fn try_resolve(&mut self, context: &mut SemanticContext) { self.as_mut_slice().try_resolve(context) }
}

impl<K: Hash + Eq, V: SemanticResolve, S: BuildHasher> SemanticResolve for IndexMap<K, V, S> {
    fn is_resolved(&self) -> bool {
        self.iter()
            .all(|(_, value)| value.is_resolved())
    }
    fn try_resolve(&mut self, context: &mut SemanticContext) {
        for (_, value) in self.iter_mut() {
            value.try_resolve(context);
        }
    }
}
