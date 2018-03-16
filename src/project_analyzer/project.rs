use std::path::PathBuf;
use indexmap::IndexMap;
use helpers::storage::SourceStorage;

//use self::module_path::ModulePath;
use super::module::Module;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    root: Module,
    // Это свойство планируется применять в реализации подключаемых библиотек (в т.ч. стандартной библиотеки СУБД)
    additional_modules: IndexMap<String, Module>,
    sources: IndexMap<PathBuf, String>,
}

impl Project {
    pub fn get_module(&self, module: Option<usize>) -> Option<&Module> {
        match module {
            Some(index) => {
                let (_, module) = self.additional_modules.get_index(index)?;
                Some(module)
            },
            None => Some(&self.root),
        }
    }
    pub fn get_module_by_name(&self, name: &str) -> Option<&Module> {
        if name == "" {
            return Some(&self.root);
        }
        self.additional_modules.get(name)
    }
    pub fn get_module_mut(&mut self, module: Option<usize>) -> Option<&mut Module> {
        match module {
            Some(index) => {
                let (_, module) = self.additional_modules.get_index_mut(index)?;
                Some(module)
            },
            None => Some(&mut self.root),
        }
    }
    pub fn get_module_by_name_mut(&mut self, name: &str) -> Option<&mut Module> {
        if name == "" {
            return Some(&mut self.root);
        }
        self.additional_modules.get_mut(name)
    }
}

impl<T> SourceStorage<ProjectIndex<T>> for Project
    where Module: SourceStorage<T> {
    type Element = <Module as SourceStorage<T>>::Element;
    fn get_element(&self, index: ProjectIndex<T>) -> Option<&Self::Element> {
        let ProjectIndex { module, index } = index;
        let module = self.get_module(module)?;
        module.get_element(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectIndex<T> {
    module: Option<usize>,
    index: T,
}

impl<T> ProjectIndex<T> {
    fn new(module: Option<usize>, index: T) -> Self {
        ProjectIndex { module, index }
    }
    fn from_vec(module: Option<usize>, indexes: Vec<T>) -> Vec<Self> {
        let mut result = Vec::with_capacity(indexes.len());
        for index in indexes {
            result.push(ProjectIndex { module, index });
        }
        result
    }
}
