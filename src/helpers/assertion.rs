pub trait Assertion<T: ?Sized = Self> {
    fn assert(&self, other: &T);
}

impl<'r, T: Assertion<R> + PartialEq<R> + ::std::fmt::Debug, R: ::std::fmt::Debug> Assertion<[R]> for [T] {
    fn assert(&self, other: &[R]) {
        let mut other_iter = other.iter();
        for item in self {
            let mut other_item = match other_iter.next() {
                Some(x) => x,
                None => return assert_eq!(self, other),
            };
            item.assert(other_item);
        }
        assert!(other_iter.next().is_none());
    }
}
