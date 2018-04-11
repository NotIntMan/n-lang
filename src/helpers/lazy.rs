// TODO Этот модуль нужен?

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lazy<T> {
    value: Option<T>,
}

impl<T> Lazy<T> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
    pub fn get<F: FnOnce() -> T>(&mut self, calculator: F) -> &mut T {
        if self.value.is_none() {
            self.value = Some(calculator());
        }
        match &mut self.value {
            Some(ref mut value) => value,
            None => unreachable!(),
        }
    }
}

impl<T> Default for Lazy<T> {
    #[inline]
    fn default() -> Self {
        Lazy {
            value: None,
        }
    }
}
