use std::thread::{
    current,
    ThreadId,
    yield_now,
};
use std::sync::{
    Mutex,
    MutexGuard,
};
use std::ops::{
    Deref,
    DerefMut,
};
use std::cmp::{
    Eq,
    PartialEq,
};
use helpers::find_index::find_index;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThreadEntityCount {
    id: ThreadId,
    count: usize,
}

impl ThreadEntityCount {
    #[inline]
    fn new(count: usize) -> Self {
        ThreadEntityCount {
            id: current().id(),
            count,
        }
    }
    #[inline]
    fn is_for_current_thread(&self) -> bool {
        current().id() == self.id
    }
    #[inline]
    fn try_inc(&mut self) -> bool {
        if self.is_for_current_thread() {
            self.count = match self.count.checked_add(1) {
                Some(x) => x,
                None => return false,
            };
            true
        } else {
            false
        }
    }
    #[inline]
    fn try_dec(&mut self) -> bool {
        if self.is_for_current_thread() {
            self.count = match self.count.checked_sub(1) {
                Some(x) => x,
                None => return false,
            };
            true
        } else {
            false
        }
    }
    #[inline]
    fn is_positive(&self) -> bool {
        self.count > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ReEntrantRWHead {
    is_poisoned: bool,
    write: Option<ThreadEntityCount>,
    readers: Vec<ThreadEntityCount>,
}

impl ReEntrantRWHead {
    fn new() -> Self {
        ReEntrantRWHead {
            is_poisoned: false,
            write: None,
            readers: Vec::new(),
        }
    }
    fn is_readers_from_one_thread(&self) -> (bool, Option<&ThreadEntityCount>) {
        if self.is_poisoned { panic!("ReEntrantRWLock was poisoned!"); }
        let mut reader = None;
        for counter in self.readers.iter() {
            if counter.is_positive() {
                match reader {
                    Some(_) => return (false, None),
                    None => reader = Some(counter),
                }
            }
        }
        (true, reader)
    }
    #[inline]
    fn is_free(&self) -> bool {
        match self.is_readers_from_one_thread() {
            (true, None) => true,
            _ => false,
        }
    }
    fn try_lock_write(&mut self) -> bool {
        if self.is_poisoned { panic!("ReEntrantRWLock was poisoned!"); }
        if let &mut Some(ref mut holder) = &mut self.write {
            return holder.try_inc();
        }
        match self.is_readers_from_one_thread() {
            (true, Some(holder)) => {
                if !holder.is_for_current_thread() {
                    return false;
                }
            }
            (true, None) => {}
            (false, _) => return false,
        }
        self.write = Some(ThreadEntityCount::new(1));
        true
    }
    fn try_release_write(&mut self) -> bool {
        match &mut self.write {
            &mut Some(ref mut holder) => {
                holder.try_dec()
            }
            &mut None => false,
        }
    }
    fn readers_for_current_thread(&mut self) -> &mut ThreadEntityCount {
        match find_index(&self.readers, |c| c.is_for_current_thread()) {
            Some(index) => &mut self.readers[index],
            None => {
                self.readers.push(ThreadEntityCount::new(0));
                self.readers.last_mut()
                    .expect("Last element was just added right before!")
            }
        }
    }
    fn try_lock_read(&mut self) -> bool {
        if self.is_poisoned { panic!("ReEntrantRWLock was poisoned!"); }
        if let &mut Some(ref mut holder) = &mut self.write {
            if !holder.is_for_current_thread() {
                return false;
            }
        }
        self.readers_for_current_thread().try_inc()
    }
    fn try_release_read(&mut self) -> bool {
        self.readers_for_current_thread().try_dec()
    }
    fn poison(&mut self) {
        self.is_poisoned = true;
    }
}

#[derive(Debug)]
pub struct ReEntrantRWLock<T: ? Sized> {
    head: Mutex<ReEntrantRWHead>,
    data: T,
}

impl<T> ReEntrantRWLock<T> {
    pub fn new(data: T) -> Self {
        ReEntrantRWLock {
            head: Mutex::new(ReEntrantRWHead::new()),
            data,
        }
    }
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T: ? Sized> ReEntrantRWLock<T> {
    fn head(&self) -> MutexGuard<ReEntrantRWHead> {
        self.head.lock()
            .expect("Head of ReEntrantRWLock was poisoned!")
    }
}

impl<'a, T: ? Sized> ReEntrantRWLock<T>
    where ReEntrantRWLock<T>: 'a {
    pub fn try_read(&'a self) -> Option<ReEntrantReadGuard<'a, T>> {
        let mut head = self.head();
        if head.try_lock_read() {
            Some(ReEntrantReadGuard {
                source: self,
            })
        } else {
            None
        }
    }
    pub fn read(&'a self) -> ReEntrantReadGuard<'a, T> {
        loop {
            match self.try_read() {
                Some(guard) => return guard,
                None => yield_now(),
            }
        }
    }
    pub fn try_write(&'a self) -> Option<ReEntrantWriteGuard<'a, T>> {
        let mut head = self.head();
        if head.try_lock_write() {
            Some(ReEntrantWriteGuard {
                source: self,
            })
        } else {
            None
        }
    }
    pub fn write(&'a self) -> ReEntrantWriteGuard<'a, T> {
        loop {
            match self.try_write() {
                Some(guard) => return guard,
                None => yield_now(),
            }
        }
    }
}

impl<T: PartialEq + ? Sized> PartialEq for ReEntrantRWLock<T> {
    fn eq(&self, other: &ReEntrantRWLock<T>) -> bool {
        let a = self.read();
        let b = other.read();
        *a == *b
    }

    fn ne(&self, other: &ReEntrantRWLock<T>) -> bool {
        let a = self.read();
        let b = other.read();
        *a != *b
    }
}

impl<T: PartialEq + ? Sized> Eq for ReEntrantRWLock<T> {}

impl<T: Clone> Clone for ReEntrantRWLock<T> {
    fn clone(&self) -> Self {
        let a = self.read();
        ReEntrantRWLock::new((*a).clone())
    }
}

#[derive(Debug)]
pub struct ReEntrantReadGuard<'a, T: ? Sized>
    where ReEntrantRWLock<T>: 'a {
    source: &'a ReEntrantRWLock<T>,
}

impl<'a, T: ? Sized> Deref for ReEntrantReadGuard<'a, T>
    where ReEntrantRWLock<T>: 'a {
    type Target = T;
    fn deref(&self) -> &T {
        &self.source.data
    }
}

impl<'a, T: ? Sized> Drop for ReEntrantReadGuard<'a, T> {
    fn drop(&mut self) {
        self.source.head().try_release_read();
    }
}

#[derive(Debug)]
pub struct ReEntrantWriteGuard<'a, T: ? Sized>
    where ReEntrantRWLock<T>: 'a {
    source: &'a ReEntrantRWLock<T>,
}

impl<'a, T: ? Sized> Deref for ReEntrantWriteGuard<'a, T>
    where ReEntrantRWLock<T>: 'a {
    type Target = T;
    fn deref(&self) -> &T {
        &self.source.data
    }
}

impl<'a, T: ? Sized> DerefMut for ReEntrantWriteGuard<'a, T>
    where ReEntrantRWLock<T>: 'a {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *(&self.source.data as *const T as *mut T)
        }
    }
}
