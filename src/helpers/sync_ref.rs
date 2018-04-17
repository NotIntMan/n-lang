use std::sync::Arc;
use std::fmt;
use helpers::re_entrant_rw_lock::{
    ReEntrantRWLock,
    ReEntrantReadGuard,
    ReEntrantWriteGuard,
};

#[derive(Clone, PartialEq, Eq)]
pub struct SyncRef<T: ? Sized> {
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

impl<T: ? Sized> SyncRef<T> {
    pub fn get_mut(&mut self) -> Option<&mut T> {
        let unique_lock = Arc::get_mut(&mut self.res)?;
        Some(unique_lock.get_mut())
    }
}

impl<'a, T: ? Sized> SyncRef<T>
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

impl<T: fmt::Debug + ? Sized> fmt::Debug for SyncRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.read();
        if f.alternate() {
            write!(f, "SyncRef of {:#?}", &*data)
        } else {
            write!(f, "SyncRef of {:?}", &*data)
        }
    }
}

impl<T: fmt::Display + ? Sized> fmt::Display for SyncRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.read();
        write!(f, "SyncRef of {}", &*data)
    }
}
