use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ID(usize);

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ID: {}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IDPull {
    counter: usize,
}

impl IDPull {
    #[inline]
    pub fn with_init_value(counter: usize) {
        IDPull { counter }
    }
    #[inline]
    pub fn new() -> Self {
        IDPull::with_init_value(0)
    }
    pub fn generate(&mut self) -> ID {
        let result = ID(self.counter);
        self.counter += 1;
        result
    }
}
