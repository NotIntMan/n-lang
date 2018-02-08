//! Группа элементов

// TODO Отладить алгоритм комбинирования

use std::mem::replace;

/// Группа элементов
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Group<T> {
    None,
    One(T),
    Many(Vec<T>),
}

use std::fmt::Debug;

impl<T> Group<T> {
    /// Выполняет копирование всех хранимых ошибок в вектор и возвращает его
    #[inline]
    pub fn extract_into_vec(&self) -> Vec<T>
        where T: Clone {
        match self {
            &Group::None => Vec::new(),
            &Group::One(ref e) => vec![(*e).clone()],
            &Group::Many(ref v) => v.clone(),
        }
    }
    pub fn append_into_slice(target: &mut [T], mut source: T) -> Option<T>
        where T: Appendable {
        for item in target.iter_mut() {
            source = match item.append(source) {
                Some(s) => s,
                None => return None,
            }
        }
        Some(source)
    }
    #[inline]
    pub fn append_or_push(target: &mut Vec<T>, item: T)
        where T: Appendable {
        let item = match Group::append_into_slice(target.as_mut_slice(), item) {
            Some(s) => s,
            None => return,
        };
        target.push(item);
    }
    /// Выполняет поглощение другой группы.
    /// После выполнения текущий объект будет содержать как свои элементы, так и элементы из переданного объекта.
    pub fn append_group(&mut self, other_error: Self)
        where T: Appendable + PartialEq + Clone + Debug {
        let result = match self {
            &mut Group::None => other_error,
            &mut Group::One(ref self_item) => {
                let self_item = self_item.clone();
                let new_vec = match other_error {
                    Group::None => return,
                    Group::One(other_item) => {
                        let mut self_item = self_item.clone();
                        if self_item == other_item { return; }
                        let other_item = match self_item.append(other_item) {
                            Some(other_item) => other_item,
                            None => return,
                        };
                        vec![self_item, other_item]
                    }
                    Group::Many(mut other_vec) => {
                        if !other_vec.contains(&self_item) {
                            Group::append_or_push(&mut other_vec, self_item.clone());
                        }
                        other_vec
                    }
                };
                Group::Many(new_vec)
            }
            &mut Group::Many(ref mut self_vec) => {
                match other_error {
                    Group::None => return,
                    Group::One(other_item) => {
                        if !self_vec.contains(&other_item) {
                            Group::append_or_push(self_vec, other_item);
                        }
                    }
                    Group::Many(mut other_vec) => {
                        for other_item in other_vec {
                            if !self_vec.contains(&other_item) {
                                Group::append_or_push(self_vec, other_item);
                            }
                        }
                    }
                }
                return;
            }
        };
        replace(self, result);
    }
}

pub trait Appendable: Sized {
    fn append(&mut self, other: Self) -> Option<Self> {
        Some(other)
    }
}

impl<T: Appendable + PartialEq + Clone + Debug> Appendable for Group<T> {
    #[inline]
    fn append(&mut self, other: Self) -> Option<Self> {
        self.append_group(other);
        None
    }
}

impl Appendable for String {}
