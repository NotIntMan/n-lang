#![allow(dead_code)]

use std::sync::Arc;
use indexmap::IndexMap;
use helpers::loud_rw_lock::LoudRwLock;
use helpers::find_index::find_index;
use lexeme_scanner::ItemPosition;
use parser_basics::StaticIdentifier;
use syntax_parser::others::StaticPath;
//use super::resolve::SemanticResolve;
use super::text_source::{
    Text,
    TextSource,
};
use super::item::{
    ProjectItem,
    ProjectItemIndex,
};
use super::error::SemanticError;

#[derive(Debug)]
pub struct Project<S> {
    source_of_source: S,
    items: IndexMap<Vec<StaticIdentifier>, LoudRwLock<ProjectItem>>,
}

#[derive(Debug, Clone)]
pub struct ProjectRef<S> {
    pub refer: Arc<LoudRwLock<Project<S>>>,
}

impl<S: TextSource> ProjectRef<S> {
    pub fn new(source_of_source: S) -> Self {
        ProjectRef {
            refer: Arc::new(LoudRwLock::new(
                Project {
                    source_of_source,
                    items: IndexMap::new(),
                },
                "Project's object was poisoned!",
            ))
        }
    }
    pub fn get_text(&self, path: &[StaticIdentifier]) -> Option<Arc<Text>> {
        let mut project = self.refer.write();
        project.source_of_source.get_text(path)
    }
    pub fn try_get_text_for_relative_path<'a>(&self, _base: &[StaticIdentifier], _path: &'a [StaticIdentifier]) -> Option<(Arc<Text>, &'a [StaticIdentifier])> {
        unimplemented!()
    }
    pub fn try_get_item_by_path(&self, base: &[StaticIdentifier], path: &[StaticIdentifier]) -> Option<ProjectItemIndex> {
        let abs_path = base.iter().chain(path.iter());
        find_index(
            &self.refer.read().items,
            |&(data_type_path, _)| abs_path.clone().eq(data_type_path.iter()),
        )
            .map(|item_id| ProjectItemIndex { item_id })
    }
    pub fn try_read_from_item<F, O>(&self, index: ProjectItemIndex, func: F) -> Option<O>
        where F: FnOnce(&Vec<StaticIdentifier>, &ProjectItem) -> O {
        let ProjectItemIndex { item_id } = index;
        let project = self.refer.read();
        let (path, item_lock) = project.items.get_index(item_id)?;
        let item = item_lock.read();
        Some(func(path, &*item))
    }
    pub fn resolve_item(&self, base: &[StaticIdentifier], path: &[StaticIdentifier], pos: ItemPosition) -> Result<ProjectItemIndex, SemanticError> {
        let (base, path) = resolve_supers(base, path, pos)?;
        if let Some(index) = self.try_get_item_by_path(base, path) {
            return Ok(index);
        }
// TODO Учесть циклические ссылки при поиске
//        let range = match path.len() {
//            0 => return Err(SemanticError::unresolved_item(
//                pos,
//                base.iter()
//                    .chain(path.iter())
//                    .cloned()
//                    .collect(),
//            )),
//            len => 1..(len-1),
//        };
//        'find_cycle: for i in range { TODO Поиск надо производить урезая путь справа налево, а не как сейчас
//            let index = match self.try_get_item_by_path(base, &path[..i]) {
//                Some(index) => index,
//                None => continue 'find_cycle,
//            };
//            let new_base = match self.try_read_from_item(index, |_, item| TODO Получение нового базового пути для продолжения поиска) {
//                Some(path) => path,
//                None => continue 'find_cycle,
//            };
//            return self.resolve_item(
//                new_base.as_slice(),
//                &path[i..],
//                pos,
//            );
//        }
        Err(SemanticError::unresolved_item(
            pos,
            base.iter()
                .chain(path.iter())
                .cloned()
                .collect(),
        ))
    }
}

pub fn resolve_supers<'a, 'b>(mut base: &'a [StaticIdentifier], mut path: &'b [StaticIdentifier], pos: ItemPosition) -> Result<(&'a [StaticIdentifier], &'b [StaticIdentifier]), SemanticError> {
    match path.get(0)
        .ok_or_else(|| SemanticError::item_name_not_specified(pos))?
        .get_text() {
        "self" => {
            path = &path[1..];
        }
        "super" => {
            'super_loop: loop {
                path = &path[1..];
                match base.len() {
                    0 => return Err(SemanticError::super_of_root(pos)),
                    length => base = &base[..(length - 1)],
                }
                if path.len() == 0 {
                    return Err(SemanticError::item_name_not_specified(pos));
                }
                if "super" != path.get(0)
                    .ok_or_else(|| SemanticError::item_name_not_specified(pos))?
                    .get_text() {
                    break 'super_loop;
                }
            }
        }
        _ => {}
    }
    Ok((base, path))
}