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

use std::fmt::{
    Display,
    Result as FResult,
    Formatter,
};

use helpers::display_list::{
    display_list,
    list_to_string,
};

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
    pub fn append_group(&mut self, other: Self)
        where T: Appendable + Clone + Display {
        trace!("Запрос на объединение {} и {}", self, other);
        let result = match self {
            &mut Group::None => {
                trace!("self оказался пустым, возвращаем other: {}", other);
                other
            },
            &mut Group::One(ref self_item) => {
                trace!("self оказался единичным");
                let mut self_item = self_item.clone();
                let new_vec = match other {
                    Group::None => {
                        trace!("а other - пустым. возвращаем self {}", self);
                        return
                    },
                    Group::One(other_item) => {
                        trace!("как и other");
                        let other_item = match self_item.append(other_item) {
                            Some(other_item) => other_item,
                            None => {
                                trace!("но self поглотил other. возвращаем self {}", self_item);
                                return
                            },
                        };
                        vec![self_item, other_item]
                    }
                    Group::Many(mut other_vec) => {
                        trace!("а other - множеством. добавляем свой элемент в него");
                        Group::append_or_push(&mut other_vec, self_item);
                        other_vec
                    }
                };
                trace!("вот что получилось: [\n{}]", list_to_string(new_vec.as_slice()));
                Group::Many(new_vec)
            }
            &mut Group::Many(ref mut self_vec) => {
                trace!("self оказался множеством");
                match other {
                    Group::None => {
                        trace!("а other - пустым. возвращаем self");
                        return
                    },
                    Group::One(other_item) => {
                        trace!("а other - единичным. добавляем его элемент в себя");
                        Group::append_or_push(self_vec, other_item);
                    }
                    Group::Many(mut other_vec) => {
                        trace!("как и other");
                        for other_item in other_vec {
                            trace!("добавляем в себя элемент из other");
                            Group::append_or_push(self_vec, other_item);
                        }
                    }
                }
                trace!("вот что получилось: [\n{}]", list_to_string(self_vec.as_slice()));
                return;
            }
        };
        replace(self, result);
        trace!("вот что получилось после замены: {}", self);
    }
}

impl<T: Display> Display for Group<T> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "[")?;
        match self {
            &Group::One(ref e) => write!(f, "{}", e)?,
            &Group::Many(ref vec) => display_list(f, vec.as_slice())?,
            _ => {},
        }
        write!(f, "]")?;
        Ok(())
    }
}

pub trait Appendable: Sized {
    #[inline]
    fn append(&mut self, other: Self) -> Option<Self> {
        Some(other)
    }
}

impl<T: Appendable + PartialEq + Clone + Display> Appendable for Group<T> {
    #[inline]
    fn append(&mut self, other: Self) -> Option<Self> {
        self.append_group(other);
        None
    }
}

impl Appendable for String {}
