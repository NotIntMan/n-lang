use std::{
    cmp::Eq,
    mem::replace,
    vec::IntoIter,
};

#[derive(Debug, Clone)]
pub struct Map<K: Eq, V> {
    pairs: Vec<(K, V)>,
}

impl<K: Eq, V> Map<K, V> {
    pub fn new() -> Self {
        Self {
            pairs: Vec::new(),
        }
    }
    pub fn iter<'s>(&'s self) -> impl Iterator<Item=(&'s K, &'s V)> {
        self.pairs.iter()
            .map(|(key, value)| (key, value))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&K, &mut V)> {
        self.pairs.iter_mut()
            .map(|(key, value)| (&*key, value))
    }
    pub fn get<'s>(&'s self, k: &impl PartialEq<K>) -> Option<&'s V> {
        for (key, value) in self.iter() {
            if *k == *key {
                return Some(value);
            }
        }
        None
    }
    pub fn get_mut<'s>(&'s mut self, k: &impl PartialEq<K>) -> Option<&'s mut V> {
        for (key, value) in self.iter_mut() {
            if *k == *key {
                return Some(value);
            }
        }
        None
    }
    pub fn has(&self, k: &impl PartialEq<K>) -> bool {
        self.get(k).is_some()
    }
    pub fn insert(&mut self, k: impl Into<K> + PartialEq<K>, value: impl Into<V>) -> Option<V> {
        let value = value.into();
        if let Some(place) = self.get_mut(&k) {
            return Some(replace(place, value));
        }
        self.pairs.push((k.into(), value));
        None
    }
}

impl<K: Eq, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}
