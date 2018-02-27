pub trait Assertion<T: ?Sized = Self> {
    fn assert(&self, other: &T);
}

impl<'r, T: Assertion<R>, R: ::std::fmt::Debug> Assertion<[R]> for [T] {
    fn assert(&self, other: &[R]) {
        let mut other_iter = other.iter();
        for item in self {
            let mut other_item = match other_iter.next() {
                Some(x) => x,
                None => panic!("Slices should have equals size"),
            };
            item.assert(other_item);
        }
        assert!(other_iter.next().is_none());
    }
}
