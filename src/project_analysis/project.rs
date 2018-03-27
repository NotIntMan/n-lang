use std::rc::Rc;
use std::fmt::Debug;
use indexmap::IndexMap;
use helpers::find_index::find_index;
use parser_basics::StaticIdentifier;
use syntax_parser::others::StaticPath;
use syntax_parser::compound_types::DataType;
use super::text_source::TextSource;
use super::context::{
    DependencyReference,
    SemanticContext,
    SemanticItemType,
};

pub trait SourceOfSource: TextSource + Debug {}

#[derive(Debug, Clone)]
pub struct Project {
    source_of_source: Rc<SourceOfSource>,
    types: IndexMap<StaticPath, ProjectItem<DataType<'static>>>,
}

pub type ProjectRef = Rc<Project>;

impl Project {
    #[inline]
    pub fn from_shared_source(source_of_source: Rc<SourceOfSource>) -> ProjectRef {
        Rc::new(Project {
            source_of_source,
            types: IndexMap::new(),
        })
    }
    #[inline]
    pub fn from_source<S: 'static + SourceOfSource>(source: S) -> ProjectRef {
        Project::from_shared_source(Rc::new(source))
    }
    pub fn resolve_item(&self, item_type: SemanticItemType, path: &[StaticIdentifier]) -> Option<DependencyReference> {
        match item_type {
            SemanticItemType::DataType => {
                let type_id = find_index(
                    &self.types,
                    |&(data_type_path, _)| data_type_path.as_slice() == path,
                )?;
                Some(DependencyReference { item_type, type_id })
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectItem<Data> {
    context: SemanticContext,
    data: Data,
}
