use std::fmt::{
    Display,
    Result as FResult,
    Formatter,
};

pub fn display_list<T: Display>(formatter: &mut Formatter, source: &[T]) -> FResult {
    let mut iter = source.iter();
    if let Some(item) = iter.next() {
        write!(formatter, "{}", item)?;
    }
    for item in iter {
        write!(formatter, ", {}", item)?;
    }
    Ok(())
}

pub fn list_to_string<T: Display>(source: &[T]) -> String {
    let mut result = String::new();
    for item in source.iter() {
        result.push_str(&format!("  {}\n", item));
    }
    result
}
