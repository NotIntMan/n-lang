use helpers::{
    ReEntrantReadGuard,
    ReEntrantRWLock,
    ReEntrantWriteGuard,
};
use std::{
    fmt,
    hash,
    sync::Arc,
};

#[derive(PartialEq, Eq)]
pub struct SyncRef<T: ?Sized> {
    res: Arc<ReEntrantRWLock<T>>,
}

impl<T> SyncRef<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        SyncRef {
            res: Arc::new(ReEntrantRWLock::new(data)),
        }
    }
}

impl<T: ?Sized> SyncRef<T> {
    pub fn get_mut(&mut self) -> Option<&mut T> {
        let unique_lock = Arc::get_mut(&mut self.res)?;
        Some(unique_lock.get_mut())
    }
    #[inline]
    pub fn is_same_ref(&self, other: &Self) -> bool {
        Arc::ptr_eq(
            &self.res,
            &other.res,
        )
    }
    #[inline]
    pub fn has_same_ref_in(&self, vec: &Vec<Self>) -> bool {
        vec.iter()
            .find(|&refer| self.is_same_ref(refer))
            .is_some()
    }
}

impl<'a, T: ?Sized> SyncRef<T>
    where ReEntrantRWLock<T>: 'a {
    #[inline]
    pub fn try_read(&'a self) -> Option<ReEntrantReadGuard<'a, T>> {
        self.res.try_read()
    }
    #[inline]
    pub fn read(&'a self) -> ReEntrantReadGuard<'a, T> {
        self.res.read()
    }
    #[inline]
    pub fn try_write(&'a self) -> Option<ReEntrantWriteGuard<'a, T>> {
        self.res.try_write()
    }
    #[inline]
    pub fn write(&'a self) -> ReEntrantWriteGuard<'a, T> {
        self.res.write()
    }
}

impl<T> Clone for SyncRef<T> {
    fn clone(&self) -> Self {
        SyncRef {
            res: self.res.clone(),
        }
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for SyncRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.read();
        if f.alternate() {
            write!(f, "SyncRef of {:#?}", &*data)
        } else {
            write!(f, "SyncRef of {:?}", &*data)
        }
    }
}

impl<T: fmt::Display + ?Sized> fmt::Display for SyncRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.read();
        write!(f, "SyncRef of {}", &*data)
    }
}

impl<T: hash::Hash> hash::Hash for SyncRef<T> {
    fn hash<H>(&self, state: &mut H)
        where H: hash::Hasher
    {
        let item = self.read();
        item.hash(state);
    }
}
