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
use super::module::{
    Module,
    ModuleRef,
};
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
    fn request_dependency(&mut self, path: StaticPath) {
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
    pub fn resolve_item(&mut self, item_type: ItemType, path: &StaticPath) -> Option<ItemRef> {
        match self.get_item(item_type, &path.path) {
            Some(x) => Some(x),
            None => {
                self.request_dependency(path.clone());
                None
            }
        }
    }
}

pub trait SemanticResolve {
    fn is_resolved(&self, context: &ResolveContext) -> bool;
    fn try_resolve(&mut self, context: &mut ResolveContext);
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
    while !queue.is_empty() {
        let mut context = ResolveContext::new(project_ref.clone());
        for (module_path, module_ref) in Extractor::new(&mut queue) {
            context.new_module(module_ref.clone());
            let mut module_is_resolved = true;
            {
                let module = module_ref.read();
                module_errors.clear();
                for item in module.items() {
                    let mut item = item.0.write();
                    if !item.is_resolved(&context) {
                        module_is_resolved = false;
                        item.try_resolve(&mut context);
                        for dependence in Extractor::new(&mut context.requested_dependencies) {
                            let mut new_module_path = dependence.path.clone();
                            for module_path_item in module_path.iter() {
                                new_module_path.insert(0, module_path_item.clone());
                            }
                            match Module::try_load(&source, dependence) {
                                Ok((new_module_ref, _)) => {
                                    project_ref.write().insert_module(new_module_path.clone(), new_module_ref.clone());
                                    next_queue.push((new_module_path, new_module_ref));
                                },
                                Err(group) => module_errors.append(&mut group.extract_into_vec()),
                            }
                        }
                    }
                }
            }
            if !module_is_resolved {
                if module_errors.is_empty() {
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
        struct Complex(double, double)
        struct Wave {
            signal: Complex,
            frequency: unsigned big integer,
        }
    ");

    source.simple_insert(vec!["complex"], "complex.n", "\
        pub struct Complex(double, double)
    ");

    match resolve(source) {
        Ok(project) => println!("Project: {:#?}", project),
        Err(errors) => {
            println!("Errors:");
            for error in errors.extract_into_vec() {
                println!("{:?}", error);
            }
        },
    }
}
