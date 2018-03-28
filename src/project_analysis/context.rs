use std::fmt;
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

#[derive(Clone)]
pub struct SemanticContext {
    module_path: Vec<StaticIdentifier>,
    project: ProjectRef,
    errors: Group<SemanticError>,
    stashed_errors: Vec<SemanticError>,
}

impl fmt::Debug for SemanticContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SemanticContext")
            .field("module_path", &self.module_path)
            .field("errors", &self.errors)
            .finish()
    }
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
        let project = self.project.refer.read();
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
    pub fn stash_errors(&mut self) {
        self.stashed_errors = self.errors.extract_into_vec();
        self.errors = Group::None;
    }
    pub fn is_errors_equal_to_stashed(&self) -> bool {
        'main_cycle: for error in self.errors.extract_into_vec() {
            for stashed_error in self.stashed_errors.iter() {
                if error == *stashed_error {
                    continue 'main_cycle;
                }
            }
            return false;
        }
        return true;
    }
}
