use parser_basics::StaticIdentifier;
use super::error::SemanticError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticItemType {
    Field,
    DataType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DependencyReference {
    pub item_type: SemanticItemType,
    pub dependency_id: usize,
    pub type_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticContext;

impl SemanticContext {
    pub fn resolve(&mut self, _item_type: SemanticItemType, path: &[StaticIdentifier]) -> Result<DependencyReference, SemanticError> {
        Err(SemanticError::UnresolvedDependency { path: path.iter().map(|r| (*r).clone()).collect() })
    }
    pub fn is_reference_resolved(&self, _refer: DependencyReference) -> bool { false }
    pub fn try_resolve_reference(&mut self, _refer: DependencyReference) {}
    pub fn error(&mut self, _error: SemanticError) {}
}
