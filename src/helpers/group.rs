//! Группа элементов

use std::mem::replace;
use helpers::IntoStatic;
use super::extract::extract;

/**
    Группа элементов

    Служит инструментом группировки элементов между собой.
    Элементы должны реализовывать типаж `Appendable` для того, чтобы можно было проводить попытки
    группировки новых элементов с ними.

    Без самостоятельной реализации типажа `Appendable` элементами поведение группы становится бессмысленным и она становится обычной обёрткой над `Vec<T>`.

    К тому же, элементы должны реализовывать типаж `Default` для быстрого их извлечения в случае само-модификации группы из `Group::One` в `Group::Many`.
*/
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group<T> {
    None,
    One(T),
    Many(Vec<T>),
}

impl<T> Group<T> {
    /// Конструирует новый Group из вектора элементов
    #[inline]
    pub fn new(items: Vec<T>) -> Self
        where T: Appendable + Default + Clone {
        let mut result = Group::None;
        for item in items {
            result.append_item(item);
        }
        return result;
    }
    /// Выполняет копирование всех хранимых ошибок в вектор и возвращает его
    #[inline]
    pub fn extract_into_vec(&self) -> Vec<T>
        where T: Clone {
        match self {
            Group::None => Vec::new(),
            Group::One(e) => vec![(*e).clone()],
            Group::Many(v) => v.clone(),
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
    /// "Добавляет" новый элемент в группу
    pub fn append_item(&mut self, item: T)
        where T: Appendable + Default {
        let new_value = match self {
            Group::None => Group::One(item),
            Group::One(self_item) => match self_item.append(item) {
                Some(item) => Group::Many(vec![extract(self_item), item]),
                None => return,
            },
            Group::Many(self_items) => {
                Group::append_or_push(self_items, item);
                return;
            }
        };
        replace(self, new_value);
    }
    /// Выполняет поглощение другой группы.
    /// После выполнения текущий объект будет содержать как свои элементы, так и элементы из переданного объекта.
    pub fn append_group(&mut self, other: Self)
        where T: Appendable + Default {
        let result = match self {
            Group::None => {
                other
            }
            Group::One(self_item) => {
                let new_vec = match other {
                    Group::None => {
                        return;
                    }
                    Group::One(other_item) => {
                        let other_item = match self_item.append(other_item) {
                            Some(other_item) => other_item,
                            None => {
                                return;
                            }
                        };
                        vec![extract(self_item), other_item]
                    }
                    Group::Many(mut other_vec) => {
                        Group::append_or_push(&mut other_vec, extract(self_item));
                        other_vec
                    }
                };
                Group::Many(new_vec)
            }
            Group::Many(self_items) => {
                match other {
                    Group::None => {
                        return;
                    }
                    Group::One(other_item) => {
                        Group::append_or_push(self_items, other_item);
                    }
                    Group::Many(mut other_vec) => {
                        for other_item in other_vec {
                            Group::append_or_push(self_items, other_item);
                        }
                    }
                }
                return;
            }
        };
        replace(self, result);
    }
}

impl<T: IntoStatic + Clone> IntoStatic for Group<T>
    where T::Result: Clone + Appendable + Default {
    type Result = Group<T::Result>;
    fn into_static(self) -> Self::Result {
        Group::new(
            self.extract_into_vec().into_static()
        )
    }
}

pub trait Appendable: Sized {
    #[inline]
    fn append(&mut self, other: Self) -> Option<Self> {
        Some(other)
    }
}

impl<T: Appendable + Default> Appendable for Group<T> {
    #[inline]
    fn append(&mut self, other: Self) -> Option<Self> {
        self.append_group(other);
        None
    }
}

impl Appendable for String {}
