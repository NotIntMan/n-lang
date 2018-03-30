use std::fmt;
use syntax_parser::compound_types::DataType;
use syntax_parser::modules::ExternalItemImport;
use super::context::SemanticContext;

#[derive(Debug, Clone)]
pub struct ProjectItem {
    context: SemanticContext,
    resolution_status: ResolutionStatus,
    body: ProjectItemBody,
}

impl ProjectItem {
    pub fn get_item_type(&self) -> ItemType {
        match &self.body {
            &ProjectItemBody::DataType(_) => ItemType::DataType,
            &ProjectItemBody::Usage(_) => ItemType::Usage,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectItemBody {
    DataType(DataType<'static>),
    Usage(ExternalItemImport<'static>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectItemIndex {
    pub item_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionStatus {
    Pending,
    InProgress,
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    DataType,
    Usage,
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ItemType::DataType => write!(f, "data type"),
            &ItemType::Usage => write!(f, "usage"),
        }
    }
}

