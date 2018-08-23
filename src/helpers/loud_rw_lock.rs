use std::{
    cmp,
    fmt,
    sync::{
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
        TryLockError,
    },
};

pub struct LoudRwLock<T: ?Sized> {
    error_msg: &'static str,
    lock: RwLock<T>,
}

impl<T> LoudRwLock<T> {
    #[inline]
    pub fn from_rw_lock(lock: RwLock<T>, error_msg: &'static str) -> Self {
        LoudRwLock {
            error_msg,
            lock,
        }
    }
    #[inline]
    pub fn new<I: Into<T>>(item: I, error_msg: &'static str) -> Self {
        LoudRwLock::from_rw_lock(RwLock::new(item.into()), error_msg)
    }
    pub fn into_inner(self) -> T {
        match self.lock.into_inner() {
            Ok(inner) => inner,
            Err(_) => panic!(self.error_msg),
        }
    }
    pub fn into_inner_safe(self) -> Option<T> {
        self.lock.into_inner().ok()
    }
}

impl<T: ?Sized> LoudRwLock<T> {
    pub fn read(&self) -> RwLockReadGuard<T> {
        match self.lock.read() {
            Ok(guard) => guard,
            Err(_) => panic!(self.error_msg),
        }
    }
    pub fn try_read(&self) -> Option<RwLockReadGuard<T>> {
        match self.lock.try_read() {
            Ok(guard) => Some(guard),
            Err(TryLockError::Poisoned(_)) => panic!(self.error_msg),
            Err(TryLockError::WouldBlock) => None,
        }
    }
    pub fn try_read_safe(&self) -> Option<RwLockReadGuard<T>> {
        self.lock.try_read().ok()
    }
    pub fn write(&self) -> RwLockWriteGuard<T> {
        match self.lock.write() {
            Ok(guard) => guard,
            Err(_) => panic!(self.error_msg),
        }
    }
    pub fn try_write(&self) -> Option<RwLockWriteGuard<T>> {
        match self.lock.try_write() {
            Ok(guard) => Some(guard),
            Err(TryLockError::Poisoned(_)) => panic!(self.error_msg),
            Err(TryLockError::WouldBlock) => None,
        }
    }
    pub fn try_write_safe(&self) -> Option<RwLockWriteGuard<T>> {
        self.lock.try_write().ok()
    }
    pub fn is_poisoned(&self) -> bool {
        self.lock.is_poisoned()
    }
    #[inline]
    pub fn is_free(&self) -> bool {
        self.try_write_safe().is_some()
    }
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.try_write_safe().is_none()
    }
    #[inline]
    pub fn is_absolutely_not_poisoned(&self) -> bool {
        self.is_free() && !self.is_poisoned()
    }
    pub fn get_mut(&mut self) -> &mut T {
        match self.lock.get_mut() {
            Ok(refer) => refer,
            Err(_) => panic!(self.error_msg),
        }
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for LoudRwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let item = self.read();
        if f.alternate() {
            write!(f, "LoudRwLock of {:#?}", &*item)
        } else {
            write!(f, "LoudRwLock of {:?}", &*item)
        }
    }
}

impl<T: ?Sized, R: ?Sized> cmp::PartialEq<LoudRwLock<R>> for LoudRwLock<T>
    where T: cmp::PartialEq<R> {
    fn eq(&self, other: &LoudRwLock<R>) -> bool {
        let left = self.read();
        let right = other.read();
        *left == *right
    }

    fn ne(&self, other: &LoudRwLock<R>) -> bool {
        let left = self.read();
        let right = other.read();
        *left != *right
    }
}

impl<T: ?Sized> cmp::Eq for LoudRwLock<T> where T: cmp::Eq {}
