use std::ops::{
    Range,
    RangeFrom,
    RangeTo,
    RangeFull,
};

use std::cmp::PartialOrd;
use std::clone::Clone;

pub trait NumRange<T = Self> {
    fn is_contains(&self, value: &T) -> bool;
    fn get_min(&self) -> Option<T>;
    fn get_max(&self) -> Option<T>;
}

pub trait Number {}

impl<T> NumRange<T> for Range<T>
    where T: PartialOrd + Clone {
    #[inline]
    fn is_contains(&self, value: &T) -> bool { self.contains(value.clone()) }
    #[inline]
    fn get_min(&self) -> Option<T> { Some(self.start.clone()) }
    #[inline]
    fn get_max(&self) -> Option<T> { Some(self.end.clone()) }
}

impl<T> NumRange<T> for RangeFrom<T>
    where T: PartialOrd + Clone {
    #[inline]
    fn is_contains(&self, value: &T) -> bool { self.contains(value.clone()) }
    #[inline]
    fn get_min(&self) -> Option<T> { Some(self.start.clone()) }
    #[inline]
    fn get_max(&self) -> Option<T> { None }
}

impl<T> NumRange<T> for RangeTo<T>
    where T: PartialOrd + Clone {
    #[inline]
    fn is_contains(&self, value: &T) -> bool { self.contains(value.clone()) }
    #[inline]
    fn get_min(&self) -> Option<T> { None }
    #[inline]
    fn get_max(&self) -> Option<T> { Some(self.end.clone()) }
}

impl<T> NumRange<T> for RangeFull
    where T: PartialOrd + Clone {
    #[inline]
    #[allow(unused_variables)]
    fn is_contains(&self, value: &T) -> bool { true }
    #[inline]
    fn get_min(&self) -> Option<T> { None }
    #[inline]
    fn get_max(&self) -> Option<T> { None }
}

impl<T> NumRange<T> for T
    where T: Number + Clone + Eq {
    #[inline]
    fn is_contains(&self, value: &T) -> bool { self == value }
    #[inline]
    fn get_min(&self) -> Option<T> { Some(self.clone()) }
    #[inline]
    fn get_max(&self) -> Option<T> { Some(self.clone()) }
}

impl Number for u8 {}
impl Number for i8 {}
impl Number for u16 {}
impl Number for i16 {}
impl Number for u32 {}
impl Number for i32 {}
impl Number for u64 {}
impl Number for i64 {}
impl Number for usize {}
impl Number for isize {}
impl Number for f32 {}
impl Number for f64 {}
