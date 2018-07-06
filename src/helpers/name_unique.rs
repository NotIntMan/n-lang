use std::{
    collections::HashSet,
    fmt::Write,
};

pub fn capitalize<'input>(input: &'input str) -> impl Iterator<Item=char> + 'input {
    let (first, last) = {
        let split_index = if input.is_empty() { 0 } else { 1 };
        input.split_at(split_index)
    };
    first.chars()
        .flat_map(char::to_uppercase)
        .chain(last.chars())
}

pub fn class_style(name: &str) -> String {
    name.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s: &&str| !s.is_empty())
        .flat_map(capitalize)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameUniquer {
    names: HashSet<String>,
}

impl NameUniquer {
    #[inline]
    pub fn new() -> Self {
        Self {
            names: HashSet::new(),
        }
    }
    pub fn add_name(&mut self, mut name: String) -> String {
        let original_length = name.len();
        let mut counter: u128 = 0;
        while self.names.contains(&name) {
            while name.len() > original_length {
                name.pop();
            }
            name.write_fmt(format_args!("_{}", counter))
                .expect("I/O error while writing in buffer string. WTF? OOM may be?");
            counter += 1;
        }
        self.names.insert(name.clone());
        name
    }
    #[inline]
    pub fn add_class_style_name(&mut self, name: &str) -> String {
        self.add_name(class_style(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        let mut n = NameUniquer::new();
        let name = String::from("result");
        assert_eq!(name, n.add_name(name.clone()));
        assert_eq!(format!("{}_0", name), n.add_name(name.clone()));
        assert_eq!(format!("{}_1", name), n.add_name(name.clone()));
    }
}
