#[derive(Debug)]
pub struct Extractor<'a, T: 'a> {
    is_reversed: bool,
    source: &'a mut Vec<T>,
}

impl<'a, T: 'a> Extractor<'a, T> {
    pub fn new(source: &'a mut Vec<T>) -> Self {
        Extractor {
            is_reversed: false,
            source,
        }
    }
}

impl<'a, T: 'a> Iterator for Extractor<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if !self.is_reversed {
            self.source.reverse();
            self.is_reversed = true;
        }
        self.source.pop()
    }
}

impl<'a, T: 'a> Drop for Extractor<'a, T> {
    fn drop(&mut self) {
        if self.is_reversed {
            self.source.reverse()
        }
    }
}
