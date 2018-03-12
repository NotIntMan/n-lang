use std::io::{
    self,
    Write,
};
use helpers::storage::{
    MatrixTextIndex,
    SourceStorage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePath(Vec<MatrixTextIndex>);

impl ModulePath {
    fn write_into<W, S>(&self, target: &mut W, text_store: &S) -> io::Result<()>
        where W: Write,
              S: SourceStorage<str, MatrixTextIndex>,
    {
        let &ModulePath(ref indexes) = self;
        let mut index_iterator = indexes.iter();
        if let Some(index) = index_iterator.next() {
            match text_store.get_element(*index) {
                Some(text) => write!(target, "{}", text)?,
                None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Text index is references into incorrect place")),
            }
        }
        for index in index_iterator {
            match text_store.get_element(*index) {
                Some(text) => write!(target, "::{}", text)?,
                None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Text index is references into incorrect place")),
            }
        }
        Ok(())
    }
    fn into_str<S>(self, text_store: &S) -> io::Result<String>
        where S: SourceStorage<str, MatrixTextIndex>
    {
        let mut result = String::with_capacity(self.0.len() * 16);
        unsafe {
            self.write_into(&mut result.as_bytes_mut(), text_store)?;
        }
        Ok(result)
    }
}
