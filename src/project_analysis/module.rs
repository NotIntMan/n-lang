use helpers::{
    Path,
    PathBuf,
    Resolve,
    SyncRef,
};
use indexmap::IndexMap;
use language::modules::{
    module,
    ModuleDefinitionItem,
    ModuleDefinitionItemAST,
};
use lexeme_scanner::Scanner;
use parser_basics::parse;
use project_analysis::{
    Item,
    ProjectContext,
    SemanticError,
    SemanticItemType,
    Text,
};
use std::{
    cmp,
    fmt,
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedModule {
    text: Arc<Text>,
    source: &'static str,
    items: Vec<ModuleDefinitionItemAST<'static>>,
}

impl UnresolvedModule {
    pub fn new(text: Arc<Text>) -> Result<Self, Vec<SemanticError>> {
        let source = Box::leak(text.text.clone().into_boxed_str());
        let tokens = match Scanner::scan(source) {
            Ok(tokens) => tokens,
            Err(error) => return SemanticError::scanner_error(error).into_err_vec(),
        };
        let items = match parse(tokens.as_slice(), module) {
            Ok(items) => items,
            Err(errors) => return Err(
                errors.extract_into_vec()
                    .into_iter()
                    .map(|error| {
                        let mut error = SemanticError::parser_error(error);
                        error.set_text(text.clone());
                        error
                    })
                    .collect()
            ),
        };
        Ok(UnresolvedModule {
            text,
            source,
            items,
        })
    }
    #[inline]
    pub fn from_text(text: Text) -> Result<Self, Vec<SemanticError>> {
        UnresolvedModule::new(Arc::new(text))
    }
}

impl Drop for UnresolvedModule {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.source as *const str as *mut str));
        }
    }
}

impl Resolve<(SyncRef<PathBuf>, SyncRef<ProjectContext>)> for UnresolvedModule {
    type Result = SyncRef<Module>;
    type Error = SemanticError;
    fn resolve(&self, ctx: &(SyncRef<PathBuf>, SyncRef<ProjectContext>)) -> Result<Self::Result, Vec<Self::Error>> {
        let context = SyncRef::new(Module::new(ctx.0.clone(), ctx.1.clone()));
        {
            let mut errors = match self.items.resolve(&context) {
                Ok(_) => Vec::new(),
                Err(mut errors) => {
                    for error in errors.iter_mut() {
                        error.set_text(self.text.clone());
                    }
                    errors
                }
            };
            let context = context.read();
            let mut item_names = Vec::new();
            for (name, item) in context.items.iter() {
                let borrowed_name = name.as_str();
                match item_names.iter().find(|&&name| name == borrowed_name) {
                    Some(_) => errors.push({
                        let mut error = SemanticError::duplicate_definition(
                            item.position,
                            name.clone(),
                            SemanticItemType::Definition,
                        );
                        error.set_text(self.text.clone());
                        error
                    }),
                    None => item_names.push(borrowed_name),
                }
            }
            if !errors.is_empty() {
                return Err(errors);
            }
        }
        Ok(context)
    }
}

#[derive(Clone)]
pub struct Module {
    items: IndexMap<String, ModuleDefinitionItem>,
    path: SyncRef<PathBuf>,
    project: SyncRef<ProjectContext>,
    imported: Vec<SyncRef<Module>>,
}

impl Module {
    #[inline]
    pub fn new(path: SyncRef<PathBuf>, project: SyncRef<ProjectContext>) -> Self {
        Module {
            items: IndexMap::new(),
            path,
            project,
            imported: Vec::new(),
        }
    }
    #[inline]
    pub fn put_item(&mut self, name: &str, value: ModuleDefinitionItem) {
        self.items.insert(name.into(), value);
    }
    #[inline]
    fn get_item_inside_module<'a>(&self, mut path: Path<'a>) -> Option<(&ModuleDefinitionItem, Path<'a>)> {
        let item = self.items.get(path.pop_left()?)?;
        Some((item, path))
    }
    pub fn get_item(&self, path: Path, search_route: &mut Vec<SyncRef<Module>>) -> Option<SyncRef<Item>> {
        match self.get_item_inside_module(path) {
            Some((item_def, rest_path)) => item_def.value.get_item(rest_path, search_route),
            None => {
                for module in self.imported.iter() {
                    if let Some(item) = module.get_item(path, search_route) {
                        return Some(item);
                    }
                }
                None
            }
        }
    }
    #[inline]
    pub fn resolve_import(&self, path: Path) -> Option<SyncRef<Item>> {
        self.project.resolve_item(path)
    }
    #[inline]
    pub fn inject_import_module(&mut self, module: SyncRef<Module>) {
        if !module.has_same_ref_in(&self.imported) {
            self.imported.push(module);
        }
    }
    #[inline]
    pub fn items(&self) -> &IndexMap<String, ModuleDefinitionItem> {
        &self.items
    }
    #[inline]
    pub fn path(&self) -> &SyncRef<PathBuf> {
        &self.path
    }
}

impl SyncRef<Module> {
    #[inline]
    pub fn put_item(&self, name: &str, value: ModuleDefinitionItem) {
        self.write().put_item(name, value)
    }
    pub fn get_item(&self, path: Path, search_route: &mut Vec<SyncRef<Module>>) -> Option<SyncRef<Item>> {
        if self.has_same_ref_in(search_route) {
            return None;
        }
        search_route.push(self.clone());
        if path.is_empty() {
            return Some(SyncRef::new(Item::module_ref(self.clone())));
        }
        self.read().get_item(path, search_route)
    }
    #[inline]
    pub fn resolve_import(&self, path: Path) -> Option<SyncRef<Item>> {
        self.read().resolve_import(path)
    }
    #[inline]
    pub fn inject_import_module(&self, module: SyncRef<Module>) {
        self.write().inject_import_module(module)
    }
    #[inline]
    pub fn project(&self) -> SyncRef<ProjectContext> { self.read().project.clone() }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Module")
            .field("items", &self.items)
            .field("path", &self.path)
            .finish()
    }
}

impl cmp::PartialEq for Module {
    fn eq(&self, other: &Module) -> bool {
        self.project.is_same_ref(&other.project)
            &&
            self.path == other.path
            &&
            self.items == other.items
    }
    fn ne(&self, other: &Module) -> bool {
        !self.project.is_same_ref(&other.project)
            ||
            self.path != other.path
            ||
            self.items != other.items
    }
}

impl cmp::Eq for Module {}
