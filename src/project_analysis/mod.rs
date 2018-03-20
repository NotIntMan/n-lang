use std::hash::{
    Hash,
    BuildHasher,
};
use std::mem::replace;
use indexmap::IndexMap;
use helpers::group::{
    Appendable,
    Group,
};
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticContext;

impl SemanticContext {
    fn resolve<'source>(_item_type: SemanticItemType, path: &[&'source str]) -> Result<DependencyReference, SemanticError<'source>> {
        Err(SemanticError::UnresolvedDependency { path: path.iter().map(|r| *r).collect() })
    }
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
    fn resolve(&mut self, context: &mut SemanticContext) -> Result<(), Group<SemanticError>>;
}

impl<T: SemanticResolve> SemanticResolve for [T] {
    fn is_resolved(&self) -> bool {
        self.iter()
            .find(|item| !(*item).is_resolved())
            .is_none()
    }
    fn resolve(&mut self, context: &mut SemanticContext) -> Result<(), Group<SemanticError>> {
        let mut errors = Group::None;
        for item in self.iter_mut() {
            if let Err(item_errors) = item.resolve(context) {
                errors.append_group(item_errors);
            }
        }
        match errors {
            Group::None => Ok(()),
            other => Err(other),
        }
    }
}

impl<T: SemanticResolve> SemanticResolve for Vec<T> {
    fn is_resolved(&self) -> bool { self.as_slice().is_resolved() }
    fn resolve(&mut self, context: &mut SemanticContext) -> Result<(), Group<SemanticError>> { self.as_mut_slice().resolve(context) }
}

impl<K: Hash + Eq, V: SemanticResolve, S: BuildHasher> SemanticResolve for IndexMap<K, V, S> {
    fn is_resolved(&self) -> bool {
        self.iter()
            .find(|&(_, value)| !value.is_resolved())
            .is_none()
    }
    fn resolve(&mut self, context: &mut SemanticContext) -> Result<(), Group<SemanticError>> {
        let mut errors = Group::None;
        for (_, value) in self.iter_mut() {
            if let Err(item_errors) = value.resolve(context) {
                errors.append_group(item_errors);
            }
        }
        match errors {
            Group::None => Ok(()),
            other => Err(other),
        }
    }
}
