use std::{
    cmp::Eq,
    iter::FromIterator,
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
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pairs: Vec::with_capacity(capacity),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item=(&K, &V)> {
        self.pairs.iter()
            .map(|(key, value)| (key, value))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&K, &mut V)> {
        self.pairs.iter_mut()
            .map(|(key, value)| (&*key, value))
    }
    pub fn get(&self, k: &impl PartialEq<K>) -> Option<&V> {
        for (key, value) in self.iter() {
            if *k == *key {
                return Some(value);
            }
        }
        None
    }
    pub fn get_mut(&mut self, k: &impl PartialEq<K>) -> Option<&mut V> {
        for (key, value) in self.iter_mut() {
            if *k == *key {
                return Some(value);
            }
        }
        None
    }
    pub fn contains_key(&self, k: &impl PartialEq<K>) -> bool {
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
    pub fn sort(&mut self)
        where K: Ord
    {
        self.pairs.sort_by(|(a_k, _), (b_k, _)| a_k.cmp(b_k))
    }
}

impl<K: Eq, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

impl<K: Eq, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<T: IntoIterator<Item=(K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let capacity = {
            let (min, max) = iter.size_hint();
            match max {
                Some(max) => min.max(max),
                None => min,
            }
        };
        let mut map = Map::with_capacity(capacity);
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}
