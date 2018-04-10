use std::mem::swap;
use helpers::group::Group;
use helpers::extractor::Extractor;
use parser_basics::Identifier;
use syntax_parser::others::StaticPath;
use super::error::SemanticError;
use super::project::{
    Project,
    ProjectRef,
};
use super::source::TextSource;
use super::module::ModuleRef;
use super::item::{
    ItemRef,
    ItemType,
};

#[derive(Debug)]
pub struct ResolveContext {
    project: ProjectRef,
    module: Option<ModuleRef>,
    requested_dependencies: Vec<StaticPath>,
    thrown_errors: Vec<SemanticError>,
}

impl ResolveContext {
    pub fn new(project: ProjectRef) -> Self {
        ResolveContext {
            project,
            module: None,
            requested_dependencies: Vec::new(),
            thrown_errors: Vec::new(),
        }
    }
    pub fn request_dependency(&mut self, path: StaticPath) {
        self.requested_dependencies.push(path)
    }
    pub fn throw_error(&mut self, error: SemanticError) {
        self.thrown_errors.push(error)
    }
    fn new_module(&mut self, module_ref: ModuleRef) {
        self.requested_dependencies.clear();
        self.thrown_errors.clear();
        self.module = Some(module_ref);
    }
    fn get_item(&self, item_type: ItemType, name: &[Identifier]) -> Option<ItemRef> {
        let arc = self.module.clone()?;
        let module = arc.read();
        module.find_item(item_type, name)
    }
    pub fn resolve_item(&self, item_type: ItemType, path: &StaticPath) -> Result<ItemRef, SemanticError> {
        match self.get_item(item_type, &path.path) {
            Some(x) => Ok(x),
            None => {
                Err(SemanticError::unresolved_item(
                    path.pos,
                    path.path.clone(),
                ))
            }
        }
    }
// Отказался т.к. не хочу параметризировать контекст
//    pub fn resolve_module(&self, path: &StaticPath) -> Result<ModuleRef, SemanticError> {
//        let project = self.project.write();
//        project.find_or_load_module()
//        unimplemented!()
//    }
}

pub trait SemanticResolve {
    fn is_resolved(&self, context: &ResolveContext) -> bool;
    fn try_resolve(&mut self, context: &mut ResolveContext);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct ModuleResolvingStatus {
    resolved: bool,
    new_resolved_items: bool,
    new_dependencies: bool,
}

pub fn resolve<S>(source: S) -> Result<ProjectRef, Group<SemanticError>>
    where S: TextSource {
    let project_ref = Project::try_init(&source)?;
    let mut errors = vec![];
    let mut queue = vec![
        (
            Vec::<Identifier<'static>>::new(),
            project_ref.read().get_root(),
        ),
    ];
    let mut next_queue = vec![];
    let mut module_errors = vec![];
//    let mut tried_dependencies = vec![];
    while !queue.is_empty() {
        let mut context = ResolveContext::new(project_ref.clone());
        for (module_path, module_ref) in Extractor::new(&mut queue) {
            println!("Resolving {:?}", module_path);
            context.new_module(module_ref.clone());
            let mut resolving_status = ModuleResolvingStatus {
                resolved: true,
                new_resolved_items: false,
                new_dependencies: false,
            };
            {
                let module = module_ref.read();
                module_errors.clear();
                for item in module.items() {
                    let mut item = item.0.write();
                    if !item.is_resolved(&context) {
                        println!("Resolving item {:?}", item.clone());
                        item.try_resolve(&mut context);
                        if item.is_resolved(&context) {
                            resolving_status.new_resolved_items = true;
                        } else {
                            resolving_status.resolved = false;
                        }
                        module_errors.append(&mut context.thrown_errors);
                        if item.is_resolved(&context) {
                            println!("Item resolved {:?}", item.clone());
                        } else {
                            println!("Item not resolved {:?}", item.clone());
                        }
                    }
                    // TODO Упростить это
                    'dep_load: for mut dependence in Extractor::new(&mut context.requested_dependencies) {
                        let mut new_module_path = dependence.path.clone();
                        for module_path_item in module_path.iter() {
                            new_module_path.insert(0, module_path_item.clone());
                        }
//                            if tried_dependencies.contains(&new_module_path) {
//                                continue 'dep_load;
//                            } else {
//                                tried_dependencies.push(new_module_path.clone());
//                            }
                        println!("Loading dependence {:?}", dependence.path);
                        // TODO Переделать механизм запроса новых зависимостей
                        // TODO Учесть ранее загруженные модули
                        match project_ref.write().find_or_load_module(&source, dependence.clone()) {
                            Ok((new_module_ref, rest_path, is_new)) => {
                                for _ in 0..rest_path.len() {
                                    let _ = new_module_path.pop();
                                    let _ = dependence.path.pop();
                                }
                                println!("Loaded {:?} ({:?})", new_module_path, dependence.path);
                                module_ref.write().put_dependency(dependence, new_module_ref.clone());
                                if is_new {
                                    next_queue.push((new_module_path, new_module_ref));
                                }
                                resolving_status.new_dependencies = true;
                            }
                            Err(group) => module_errors.append(&mut group.extract_into_vec()),
                        }
                    }
                }
                for error in module_errors.iter_mut() {
                    error.set_text(module.text());
                }
            }
            if resolving_status.resolved {
                println!("Module is resolved {:?}", module_path);
            } else {
                println!("Module is not resolved {:?}", module_path);
                if resolving_status.new_resolved_items || resolving_status.new_dependencies {
                    next_queue.push((module_path, module_ref));
                } else {
                    errors.append(&mut module_errors);
                }
            }
        }
        swap(&mut queue, &mut next_queue);
    }
    if errors.is_empty() {
        Ok(project_ref)
    } else {
        Err(Group::Many(errors))
    }
}

#[test]
fn do_it() {
    use project_analysis::source::HashMapSource;

    let mut source = HashMapSource::new();

    source.simple_insert(vec![], "index.n", "\
        use complex::Complex;
        struct Wave {
            signal: Complex,
            frequency: unsigned big integer,
        }
    ");

    source.simple_insert(vec!["complex"], "complex.n", "\
        pub struct Complex(double, double)
        pub struct SuperComplex(double, double)
    ");

    match resolve(source) {
        Ok(project) => println!("Project: {:#?}", project),
        Err(errors) => {
            println!("Errors:");
            for error in errors.extract_into_vec() {
                println!("{}", error);
            }
        },
    }
}
