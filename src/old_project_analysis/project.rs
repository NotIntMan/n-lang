#![allow(dead_code)]

use std::sync::Arc;
use std::fmt;
use indexmap::IndexMap;
use helpers::find_index::find_index;
use helpers::group::Group;
use helpers::into_static::IntoStatic;
use helpers::loud_rw_lock::LoudRwLock;
use parser_basics::StaticIdentifier;
use syntax_parser::modules::{
    DataTypeDefinition,
    ExternalItemImport,
    ModuleDefinitionItem,
    ModuleDefinitionValue,
};
use syntax_parser::others::StaticPath;
use syntax_parser::compound_types::DataType;
use super::error::SemanticError;
use super::text_source::{
    Text,
    TextSourceWithDebug,
};
use super::context::{
    ItemReference,
    SemanticContext,
};
use super::path_resolver::PathResolver;
use super::resolve::SemanticResolve;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    DataType,
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ItemType::DataType => write!(f, "data type"),
        }
    }
}

impl ItemType {
    const ALL: [Self; 1] = [
        ItemType::DataType,
    ];
}

#[derive(Debug)]
pub struct Project {
    source_of_source: Box<TextSourceWithDebug>,
    types: IndexMap<Vec<StaticIdentifier>, LoudRwLock<ProjectItem<DataType<'static>>>>,
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
    pub fn resolve_item(&self, item_type: ItemType, base: &[StaticIdentifier], path: &StaticPath) -> Result<ItemReference, SemanticError> {
        let absolute_path: Vec<_> = PathResolver::new(base, path)?
            .map(|item| (*item).clone())
            .collect();
        match item_type {
            ItemType::DataType => {
                let item_id = match find_index(
                    &self.types,
                    |&(data_type_path, _)| absolute_path.iter().eq(data_type_path.iter()),
                ) {
                    Some(index) => index,
                    None => return Err(SemanticError::unresolved_item(path.pos, absolute_path)),
                };
                Ok(ItemReference { item_type, item_id })
            }
        }
    }
    pub fn is_item_resolved(&self, refer: ItemReference) -> bool {
        match refer.item_type {
            ItemType::DataType => {
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
    pub fn items_count(&self, item_type: ItemType) -> usize {
        match item_type {
            ItemType::DataType => self.types.len(),
        }
    }
    pub fn get_text(&self, item_type: ItemType, item_id: usize) -> Option<Arc<Text>> {
        match item_type {
            ItemType::DataType => {
                let (_, item) = self.types.get_index(item_id)?;
                let item = item.read();
                Some(item.text.clone())
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
    text: Arc<Text>,
    context: SemanticContext,
    resolution_status: ResolutionStatus,
    data: Data,
}

impl<Data: SemanticResolve> ProjectItem<Data> {
    #[inline]
    fn new(text: Arc<Text>, module_path: Vec<StaticIdentifier>, project: ProjectRef, data: Data) -> Self {
        ProjectItem {
            text,
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
        let (text, items) = project.source_of_source.try_load_module(path)?;
        let items = items.into_static();
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
                        ProjectItem::new(text.clone(), path.clone(), self.clone(), body),
                        "Item's object was poisoned!",
                    );
                    // Добавляем в путь имя элемента.
                    path.push(name);
                    // И под этим путём-именем записываем в регистр.
                    project.types.insert(path, item);
                }
                ModuleDefinitionValue::Import(ExternalItemImport { path: _, tail: _ }) => {}
                _ => unimplemented!(),
            }
        }
        Ok(())
    }
    pub fn try_resolve_module(&self, item_type: ItemType, id: usize) -> ResolveResult {
        let project = self.refer.read();
        match item_type {
            ItemType::DataType => {
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
    pub fn items_count(&self, item_type: ItemType) -> usize {
        let project = self.refer.read();
        project.items_count(item_type)
    }
    pub fn get_text(&self, item_type: ItemType, item_id: usize) -> Option<Arc<Text>> {
        let project = self.refer.read();
        project.get_text(item_type, item_id)
    }
}

#[test]
fn do_it() {
    use helpers::write_pad::display;
    use project_analysis::text_source::HashMapSource;
    let mut sources = HashMapSource::new();
    sources.simple_insert(
        vec![],
        "index.n",
        "\
        use ext::X;\n \
        struct A ( boolean, X )",
    );
    sources.simple_insert(
        vec!["ext"],
        "ext.n",
        "\
        use ext::X;\n \
        struct A ( boolean, X )",
    );
    let project = Project::from_source(sources);
    println!("{:#?}", project.try_load_dependence(&vec![]));
    for &item_type in ItemType::ALL.iter() {
        for item_id in 0..project.items_count(item_type) {
            println!("Resolving {} #{}", item_type, item_id);
            match project.try_resolve_module(item_type, 0) {
                ResolveResult::WrongId => println!("There is no {} with id {}", item_type, item_id),
                ResolveResult::Errors(errors) => {
                    println!("Got errors:");
                    for error in errors {
                        println!("{}", display(|w|
                            error.write_display(w, project.get_text(item_type, item_id))
                        ));
                    }
                }
                ResolveResult::Stuck => println!("It seems to resolving is stuck"),
                ResolveResult::Resolved => println!("Resolving succeed!"),
            }
        }
    }
    println!("{:#?}", project.try_resolve_module(ItemType::DataType, 0));
    println!("{:#?}", project);
}
