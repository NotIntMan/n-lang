use parser_basics::StaticIdentifier;
use super::error::SemanticError;
use super::project::ProjectRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticItemType {
    Field,
    DataType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DependencyReference {
    pub item_type: SemanticItemType,
    pub type_id: usize,
}

#[derive(Debug, Clone)]
pub struct SemanticContext {
    module_path: Vec<StaticIdentifier>,
    project: ProjectRef,
}

impl SemanticContext {
    pub fn resolve(&mut self, _item_type: SemanticItemType, path: &[StaticIdentifier]) -> Result<DependencyReference, SemanticError> {
        Err(SemanticError::UnresolvedDependency { path: path.iter().map(|r| (*r).clone()).collect() })
    }
    pub fn is_reference_resolved(&self, _refer: DependencyReference) -> bool { false }
    pub fn try_resolve_reference(&mut self, _refer: DependencyReference) {}
    pub fn error(&mut self, _error: SemanticError) {}
}
