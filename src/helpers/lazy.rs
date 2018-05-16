use std::mem::replace;

pub trait Lazy {
    type Item;
    fn get(&self) -> Option<&Self::Item>;
    fn get_mut(&mut self) -> Option<&mut Self::Item>;
    fn set(&mut self, value: Self::Item);

    fn init_if_not<'a>(&'a mut self, init: impl FnOnce() -> Self::Item) -> &'a mut Self::Item {
        if self.get().is_none() {
            self.set(init());
        }
        self.get_mut()
            .expect("Lazy-value cannot be uninitialized right after initialize")
    }
}

impl<T> Lazy for Option<T> {
    type Item = T;
    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        match self {
            Some(item) => Some(item),
            None => None,
        }
    }
    #[inline]
    fn get_mut(&mut self) -> Option<&mut Self::Item> {
        match self {
            Some(item) => Some(item),
            None => None,
        }
    }
    #[inline]
    fn set(&mut self, value: Self::Item) {
        replace(self, Some(value));
    }
}
