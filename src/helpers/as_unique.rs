use std::hash::Hash;
use indexmap::IndexMap;

// TODO Semantic error on fail
pub fn as_unique<K: Eq + Hash, V>(vec: Vec<(K, V)>) -> Result<IndexMap<K, V>, ()> {
    let mut result = IndexMap::new();
    for (key, value) in vec {
        if result.insert(key, value).is_some() {
            return Err(());
        }
    }
    Ok(result)
}
