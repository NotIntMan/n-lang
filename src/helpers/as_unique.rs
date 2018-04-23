use std::hash::Hash;
use indexmap::IndexMap;
use parser_basics::Identifier;

pub fn as_unique<K: Eq + Hash + Clone, V>(vec: Vec<(K, V)>) -> Result<IndexMap<K, V>, K> {
    let mut result = IndexMap::new();
    for (key, value) in vec {
        if result.insert(key.clone(), value).is_some() {
            return Err(key);
        }
    }
    Ok(result)
}

pub fn as_unique_identifier<'source, T>(vec: Vec<(Identifier<'source>, T)>) -> Result<IndexMap<String, T>, Identifier<'source>> {
    let mut result = IndexMap::new();
    for (key, value) in vec {
        if result.insert(key.text().to_string(), value).is_some() {
            return Err(key);
        }
    }
    Ok(result)
}
