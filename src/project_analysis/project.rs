use std::sync::{
    Arc,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
    TryLockError,
};
use indexmap::IndexMap;
use helpers::find_index::find_index;
use helpers::group::Group;
use helpers::into_static::IntoStatic;
use lexeme_scanner::Scanner;
use parser_basics::{
    parse,
    StaticIdentifier,
};
use syntax_parser::modules::{
    DataTypeDefinition,
    module,
    ModuleDefinitionItem,
    ModuleDefinitionValue,
};
use syntax_parser::others::StaticPath;
use syntax_parser::compound_types::DataType;
use super::error::SemanticError;
use super::text_source::TextSourceWithDebug;
use super::context::{
    DependencyReference,
    SemanticContext,
};
use super::path_resolver::PathResolver;

#[derive(Debug)]
pub struct Project {
    source_of_source: Box<TextSourceWithDebug>,
    types: IndexMap<Vec<StaticIdentifier>, ProjectItem<DataType<'static>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependenceType {
    DataType,
}

impl Project {
    #[inline]
    pub fn from_boxed_source(source_of_source: Box<TextSourceWithDebug>) -> ProjectRef {
        ProjectRef {
            refer: Arc::new(RwLock::new(Project {
                source_of_source,
                types: IndexMap::new(),
            }))
        }
    }
    #[inline]
    pub fn from_source<S: 'static + TextSourceWithDebug>(source: S) -> ProjectRef {
        Project::from_boxed_source(Box::new(source))
    }
    pub fn resolve_dependence(&self, item_type: DependenceType, base: &[StaticIdentifier], path: &StaticPath) -> Result<DependencyReference, SemanticError> {
        let absolute_path: Vec<_> = PathResolver::new(base, path)?
            .map(|item| (*item).clone())
            .collect();
        match item_type {
            DependenceType::DataType => {
                let item_id = match find_index(
                    &self.types,
                    |&(data_type_path, _)| absolute_path.iter().eq(data_type_path.iter()),
                ) {
                    Some(index) => index,
                    None => return Err(SemanticError::unresolved_dependency(path.pos, absolute_path)),
                };
                Ok(DependencyReference { item_type, item_id })
            }
        }
    }
    pub fn is_dependence_resolved(&self, refer: DependencyReference) -> bool {
        match refer.item_type {
            DependenceType::DataType => {
                match self.types.get_index(refer.item_id) {
                    Some((_, ref item)) => item.resolution_status == ResolutionStatus::Resolved,
                    None => false,
                }
            }
        }
    }
    fn try_load_dependence(&mut self, path: &[StaticIdentifier]) -> Result<Vec<ModuleDefinitionItem>, Group<SemanticError>> {
        let text = match self.source_of_source.get_text(path) {
            Some(text) => text,
            None => return Err(Group::One(SemanticError::unresolved_dependency(
                Default::default(),
                path.to_vec(),
            ))),
        };
        let tokens = match Scanner::scan(&text) {
            Ok(tokens) => tokens,
            Err(error) => return Err(Group::One(SemanticError::scanner_error(error))),
        };
        match parse(&tokens, module) {
            Ok(items) => Ok(items.into_static()),
            Err(error_group) => Err(Group::new(
                error_group.extract_into_vec().into_iter()
                    .map(|item| SemanticError::parser_error(item))
                    .collect()
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionStatus {
    Pending,
    InProgress,
    Resolved,
}

#[derive(Debug, Clone)]
pub struct ProjectItem<Data> {
    context: SemanticContext,
    resolution_status: ResolutionStatus,
    data: Data,
}

impl<Data> ProjectItem<Data> {
    #[inline]
    fn new(module_path: Vec<StaticIdentifier>, project: ProjectRef, data: Data) -> Self {
        ProjectItem {
            context: SemanticContext::new(module_path, project),
            resolution_status: ResolutionStatus::Pending,
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectRef {
    refer: Arc<RwLock<Project>>,
}

impl ProjectRef {
    pub fn try_read(&self) -> Option<RwLockReadGuard<Project>> {
        match self.refer.try_read() {
            Ok(guard) => Some(guard),
            Err(TryLockError::Poisoned(_)) => panic!("Project's object was poisoned!"),
            Err(TryLockError::WouldBlock) => None,
        }
    }
    pub fn read(&self) -> RwLockReadGuard<Project> {
        match self.refer.read() {
            Ok(guard) => guard,
            Err(_) => panic!("Project's object was poisoned!"),
        }
    }
    pub fn try_write(&self) -> Option<RwLockWriteGuard<Project>> {
        match self.refer.try_write() {
            Ok(guard) => Some(guard),
            Err(TryLockError::Poisoned(_)) => panic!("Project's object was poisoned!"),
            Err(TryLockError::WouldBlock) => None,
        }
    }
    pub fn write(&self) -> RwLockWriteGuard<Project> {
        match self.refer.write() {
            Ok(guard) => guard,
            Err(_) => panic!("Project's object was poisoned!"),
        }
    }
    pub fn try_load_dependence(&self, path: &[StaticIdentifier]) -> Result<(), Group<SemanticError>> {
        let mut project = self.write();
        let items = project.try_load_dependence(path)?.into_static();
        for item in items {
            let ModuleDefinitionItem {
                public: _, // TODO Учесть экспорты
                attributes: _, // TODO Учесть аттрибуты
                value,
            } = item;
            match value {
                ModuleDefinitionValue::DataType(DataTypeDefinition { name, body }) => {
                    let mut path = path.to_vec();
                    path.push(name);
                    let item = ProjectItem::new(path.clone(), self.clone(), body);
                    project.types.insert(path, item);
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }
}

#[test]
fn do_it() {
    use std::collections::HashMap;
//    use parser_basics::Identifier;
    let mut sources = HashMap::new();
    sources.insert(
        vec![],
        "\
            struct A ( boolean, X )\
        ".to_string(),
    );
    let project = Project::from_source(sources);
    println!("{:#?}", project.try_load_dependence(&vec![]));
    println!("{:#?}", project);
}
