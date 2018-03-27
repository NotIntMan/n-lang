use helpers::group::Group;
use parser_basics::StaticIdentifier;
use syntax_parser::others::StaticPath;
use super::error::SemanticError;
use super::project::ProjectRef;
use super::project::DependenceType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticItemType {
    Field,
    DataType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DependencyReference {
    pub item_type: DependenceType,
    pub item_id: usize,
}

#[derive(Debug, Clone)]
pub struct SemanticContext {
    module_path: Vec<StaticIdentifier>,
    project: ProjectRef,
    errors: Group<SemanticError>,
}

impl SemanticContext {
    #[inline]
    pub fn new(module_path: Vec<StaticIdentifier>, project: ProjectRef) -> Self {
        SemanticContext {
            module_path,
            project,
            errors: Group::None,
        }
    }
    #[inline]
    pub fn resolve_dependence(&mut self, item_type: DependenceType, path: &StaticPath) -> Result<DependencyReference, SemanticError> {
        let project = self.project.read();
        project.resolve_dependence(item_type, self.module_path.as_slice(), path)
    }
    #[inline]
    pub fn is_dependence_resolved(&self, _refer: DependencyReference) -> bool { false }
    #[inline]
    pub fn error(&mut self, error: SemanticError) {
        self.errors.append_item(error)
    }
    #[inline]
    pub fn get_errors(&self) -> Vec<SemanticError> {
        self.errors.extract_into_vec()
    }
    #[inline]
    pub fn clear_errors(&mut self) {
        self.errors = Group::None;
    }
}
