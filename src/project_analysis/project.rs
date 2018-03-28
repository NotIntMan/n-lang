use std::sync::Arc;
use indexmap::IndexMap;
use helpers::find_index::find_index;
use helpers::group::Group;
use helpers::into_static::IntoStatic;
use helpers::loud_rw_lock::LoudRwLock;
use parser_basics::StaticIdentifier;
use syntax_parser::modules::{
    DataTypeDefinition,
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
use super::resolve::SemanticResolve;

#[derive(Debug)]
pub struct Project {
    source_of_source: Box<TextSourceWithDebug>,
    types: IndexMap<Vec<StaticIdentifier>, LoudRwLock<ProjectItem<DataType<'static>>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependenceType {
    DataType,
}

impl Project {
    #[inline]
    pub fn from_boxed_source(source_of_source: Box<TextSourceWithDebug>) -> ProjectRef {
        ProjectRef {
            refer: Arc::new(LoudRwLock::new(Project {
                source_of_source,
                types: IndexMap::new(),
            }, "Project's object was poisoned!"))
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
                    Some((_, item)) => match item.try_read_safe() {
                        Some(ref item) => item.resolution_status == ResolutionStatus::Resolved,
                        None => false,
                    }
                    None => false,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionStatus {
    Pending,
    InProgress,
    Resolved,
}

#[derive(Debug)]
pub struct ProjectItem<Data: SemanticResolve> {
    context: SemanticContext,
    resolution_status: ResolutionStatus,
    data: Data,
}

impl<Data: SemanticResolve> ProjectItem<Data> {
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
    pub refer: Arc<LoudRwLock<Project>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolveResult {
    WrongId,
    Errors(Vec<SemanticError>),
    Stuck,
    Resolved,
}

impl ProjectRef {
    pub fn try_load_dependence(&self, path: &[StaticIdentifier]) -> Result<(), Group<SemanticError>> {
        let mut project = self.refer.write();
        let items = project.source_of_source.try_load_module(path)?.into_static();
        for item in items {
            let ModuleDefinitionItem {
                public: _, // TODO Учесть экспорты
                attributes: _, // TODO Учесть аттрибуты
                value,
            } = item;
            match value {
                ModuleDefinitionValue::DataType(DataTypeDefinition { name, body }) => {
                    // Складываем путь в вектор
                    let mut path = path.to_vec();
                    // Конструируем ProjectItem и передаём ему ПУТЬ МОДУЛЯ.
                    let item = LoudRwLock::new(
                        ProjectItem::new(path.clone(), self.clone(), body),
                        "Item's object was poisoned!",
                    );
                    // Добавляем в путь имя элемента.
                    path.push(name);
                    // И под этим путём-именем записываем в регистр.
                    project.types.insert(path, item);
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }
    pub fn try_resolve_module(&self, item_type: DependenceType, id: usize) -> ResolveResult {
        let project = self.refer.read();
        match item_type {
            DependenceType::DataType => {
                let mut item = match project.types.get_index(id) {
                    Some((_, item)) => item.write(),
                    None => return ResolveResult::WrongId,
                };
                match item.resolution_status {
                    ResolutionStatus::Pending | ResolutionStatus::InProgress
                    => {
                        let item = &mut *item;
                        if item.data.is_resolved(&item.context) {
                            item.resolution_status = ResolutionStatus::Resolved;
                            return ResolveResult::Resolved;
                        }
                        if item.resolution_status == ResolutionStatus::InProgress {
                            item.context.stash_errors();
                        }
                        item.data.try_resolve(&mut item.context);
                        let errors = item.context.get_errors();
                        if errors.len() == 0 {
                            item.resolution_status = ResolutionStatus::Resolved;
                            return ResolveResult::Resolved;
                        }
                        if item.resolution_status == ResolutionStatus::InProgress
                            && item.context.is_errors_equal_to_stashed() {
                            return ResolveResult::Stuck;
                        }
                        ResolveResult::Errors(errors)
                    }
                    ResolutionStatus::Resolved => ResolveResult::Resolved,
                }
            }
        }
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
    println!("{:#?}", project.try_resolve_module(DependenceType::DataType, 0));
    println!("{:#?}", project);
}
