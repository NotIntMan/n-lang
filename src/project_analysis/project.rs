//use std::collections::HashMap;
//use std::sync::Arc;
//use helpers::group::Group;
//use helpers::re_entrant_rw_lock::ReEntrantRWLock;
////use parser_basics::StaticIdentifier;
//use syntax_parser::others::{
//    Path,
//    StaticPath,
//};
//use super::module::{
//    ModuleRef,
//};
//use super::error::SemanticError;
//use super::item::ItemRef;



//// TODO Написать свою структуру ссылок для обеспечения безопасности утечек памяти.
//pub type ProjectRef = Arc<ReEntrantRWLock<Project>>;
//
//pub type ModulePath = Vec<StaticIdentifier>;
//pub type ModulePathSlice = [StaticIdentifier];
//

//    pub fn try_init<S: TextSource>(source: &S) -> Result<SyncRef<Project>, Vec<SemanticError>> {
//        let mut project = Project::new();
//        project.find_or_load_module(source, Path::new("", "::"))?;
//        Ok(SyncRef::new(project))
//    }
//    pub fn insert_module(&mut self, path: ModulePath, module: ModuleRef) {
//        self.modules.insert(path, module);
//    }
//    pub fn get_module(&self, path: Path) -> Option<SyncRef<Module>> {
//        for (name, module) in self.modules.iter() {
//            if name.as_path() == path {
//                return Some(module.clone());
//            }
//            None
//        }
//    }
//    #[inline]
//    pub fn get_root(&self) -> SyncRef<Module> {
//        self.get_module(Path::new("", "::"))
//            .expect("Project do not contain root module")
//    }
//    // TODO Перенести try_load сюда для удобства чтения из источника и хранилища модулей
//    fn _find_or_load_module<S: TextSource>(&mut self, source: &S, path: Path) -> Option<Result<(SyncRef<Module>, bool), Vec<SemanticError>>> {
//        if let Some(module) = self.get_module(path) {
//            return Some(Ok((module, false)));
//        }
//        let text = source.get_text(path)?;
//        match Module::try_parse(text) {
//            Ok(module) => {
//                self.insert_module(path.to_vec(), module.clone());
//                Some(Ok((module, true)))
//            },
//            Err(group) => Some(Err(group)),
//        }
//    }
//    pub fn find_or_load_module<S: TextSource>(&mut self, source: &S, path: Path) -> Result<(SyncRef<Module>, Path, bool), Vec<SemanticError>> {
//        let path_len = path.path.len();
//        for i in 0..=path_len {
//            let module_path_len = path_len - i;
//            let module_path = &path.path[..module_path_len];
//            match self._find_or_load_module(source, module_path) {
//                Some(Ok((module, new_flag))) => {
//                    let rest_path = path.path[module_path_len..].to_vec();
//                    return Ok((module, rest_path, new_flag));
//                }
//                Some(Err(errors)) => return Err(errors),
//                _ => {}
//            }
//        }
//        Err(vec![SemanticError::unresolved_item(
//            path.pos,
//            path.path,
//        )])
//    }
//    pub fn find_or_load_item<S: TextSource>(&mut self, source: &S, path: StaticPath) -> Result<ItemRef, Group<SemanticError>> {
//        let (module, item_path, _) = self.find_or_load_module(source, path.clone())?;
//        match module.find_item(item_path.as_slice()) {
//            Some(item) => Ok(item),
//            None => Err(Group::One(SemanticError::unresolved_item(
//                path.pos,
//                path.path,
//            ))),
//        }
//    }
//}
