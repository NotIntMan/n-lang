use std::sync::Arc;
use std::fmt;
use std::cmp;
use indexmap::IndexMap;
//use helpers::group::Group;
//use helpers::re_entrant_rw_lock::ReEntrantRWLock;
use helpers::resolve::Resolve;
use helpers::sync_ref::SyncRef;
use helpers::path::{
    Path,
    PathBuf,
};
use lexeme_scanner::Scanner;
use parser_basics::parse;
use syntax_parser::modules::{
    module,
    ModuleDefinitionItem,
    ModuleDefinitionItemAST,
};
//use syntax_parser::others::StaticPath;
use project_analysis::{
    Item,
    Text,
    SemanticItemType,
    SemanticError,
    ProjectContext,
};

//use super::item::{
//    ItemBody,
//    ItemRef,
//};
//use super::error::SemanticError;
//
#[derive(Debug, Clone, PartialEq, Eq)]
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
            Err(error) => return Err(vec![SemanticError::scanner_error(error)]),
        };
        let items = match parse(tokens.as_slice(), module) {
            Ok(items) => items,
            Err(errors) => return Err(
                errors.extract_into_vec()
                    .into_iter()
                    .map(|error| SemanticError::parser_error(error))
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
    type Result = Module;
    type Error = SemanticError;
    fn resolve(&self, ctx: &mut (SyncRef<PathBuf>, SyncRef<ProjectContext>)) -> Result<Self::Result, Vec<Self::Error>> {
        let mut context = Module::new(ctx.0.clone(), ctx.1.clone());
        {
            let mut errors = match self.items.resolve(&mut context) {
                Ok(_) => Vec::new(),
                Err(mut errors) => {
                    for error in errors.iter_mut() {
                        error.set_text(self.text.clone());
                    }
                    errors
                }
            };
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
}

impl SyncRef<Module> {
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

//#[derive(Debug, Clone, PartialEq, Eq)]
//pub struct ModuleRef(pub Arc<ReEntrantRWLock<Module>>);
//
//impl Module {
//    pub fn try_parse(text: Arc<Text>) -> Result<ModuleRef, Group<SemanticError>> {
//        let tokens = match Scanner::scan(&text.text) {
//            Ok(tokens) => tokens,
//            Err(error) => return Err(Group::One(SemanticError::scanner_error(error))),
//        };
//        match parse(&tokens, module) {
//            Ok(items) => Ok(Module::from_def(text.clone(), items)),
//            Err(error_group) => Err(Group::new(
//                error_group.extract_into_vec().into_iter()
//                    .map(|item| SemanticError::parser_error(item))
//                    .collect()
//            )),
//        }
//    }
//    pub fn from_def(text: Arc<Text>, items: Vec<ModuleDefinitionItem>) -> ModuleRef {
//        let module_ref = ModuleRef(Arc::new(ReEntrantRWLock::new(Module {
//            text,
//            items: Vec::with_capacity(items.len()),
//            injected_dependencies: Vec::new(),
//        })));
//        {
//            let mut module = module_ref.0.write();
//            for item in items {
//                module.items.push(ItemRef::from_def(item))
//            }
//        }
//        module_ref
//    }
//    pub fn items(&self) -> &[ItemRef] {
//        &self.items
//    }
//    pub fn text(&self) -> Arc<Text> {
//        self.text.clone()
//    }
//}
//
//impl ModuleRef {
//    pub fn put_dependency(&self, path: StaticPath, dependency: &ModuleRef, errors: &mut Vec<SemanticError>) -> bool {
//        let module = self.0.read();
//        if module.injected_dependencies.contains(&path) {
//            return false;
//        }
//        println!("Putting {:?} into module {:?}", path.path, module.text.name);
//        for item in module.items.iter() {
//            match item.put_dependency(&path, dependency) {
//                Ok(()) => {}
//                Err(err) => errors.push(err),
//            }
//        }
//        {
//            let mut module = self.0.write();
//            module.injected_dependencies.push(path);
//        }
//        true
//    }
//    pub fn find_item(&self, name: &[Identifier]) -> Option<ItemRef> {
//        let module = self.0.read();
//        println!("Finding item {:?} in module {:?}", name, module.text.name);
//        if name.is_empty() {
//            return Some(ItemRef::from_body(ItemBody::ModuleReference { module: self.clone() }));
//        }
//        for item in module.items.iter() {
//            if let Some(item_ref) = item.find_item(name) {
//                return Some(item_ref);
//            }
//        }
//        None
//    }
//}
//
//#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
//pub struct ModuleId {
//    pub module_id: usize,
//}
//
