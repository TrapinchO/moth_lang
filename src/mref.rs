use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use crate::backend::value::Value;

// stands for MothReference
#[derive(Clone)]
pub struct MRef<T>(Rc<UnsafeCell<T>>);
impl<T> MRef<T> {
    pub fn new(val: T) -> Self {
        Self(Rc::new(UnsafeCell::new(val)))
    }

    pub fn read<V: 'static>(&self, f: impl FnOnce(&T) -> V) -> V {
        unsafe { f(&*self.0.get()) }
    }

    pub fn write(&mut self, val: T) {
        unsafe {
            *(self.0.get()) = val;
        }
    }
}

impl<T> From<T> for MRef<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: PartialEq> PartialEq for MRef<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.0.get() == *other.0.get() }
    }
}

impl<T: Debug> Debug for MRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "Mref({:?})", &*self.0.get()) }
    }
}


pub type MMap<T> = MRef<HashMap<String, T>>;
impl<T: Clone> MMap<T> {
    pub fn insert(&mut self, key: String, val: T) {
        unsafe {
            let map = &mut *self.0.get();
            map.insert(key, val);
        }
    }

    pub fn get(&self, key: &String) -> Option<&T> {
        unsafe {
            let map = &*self.0.get();
            map.get(key)
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (String, T)> {
        MMapIter::new(self.clone())
    }
}

struct MMapIter<T> {
    idx: usize,
    len: usize,
    keys: Vec<String>,
    map: MMap<T>,
}
impl<T> MMapIter<T> {
    fn new(map: MMap<T>) -> Self {
        Self {
            idx: 0,
            len: map.read(|m| m.len()),
            keys: map.read(|m| m.keys().cloned().collect::<Vec<_>>()),
            map,
        }
    }
}
impl<T: Clone> Iterator for MMapIter<T> {
    type Item = (String, T);
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let key = self.keys[self.idx].clone();
        let item = self.map.get(&key).unwrap();
        self.idx += 1;
        Some((key, item.clone()))
    }
}


pub type MList = MRef<Vec<Value>>;
impl MList {
    pub fn modify(&mut self, idx: usize, val: Value) {
        unsafe {
            let ls = &mut *self.0.get();
            ls[idx] = val;
        }
    }

    // not necessary for now
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.read(|l| l.len())
    }

    pub fn iter(&self) -> impl Iterator<Item = Value> {
        MListIter::new(self.clone())
    }

    // checks whether it is in the possible range (even if negative)
    // and returns it as a positive index
    // NOTE: for future me, this is also used for indexing strings
    pub fn check_index(idx: i32, length: usize) -> Option<usize> {
        if idx < 0 {
            if (idx.unsigned_abs() as usize) > length { None }
            else { Some(length - (idx.unsigned_abs() as usize)) }
        } else {
            if (idx as usize) >= length { None }
            else { Some(idx as usize) }
        }
    }
}
struct MListIter {
    idx: usize,
    len: usize,
    ls: MList,
}
impl MListIter {
    fn new(ls: MList) -> Self {
        let len = ls.read(|l| l.len());
        Self { ls, len, idx: 0 }
    }
}
impl Iterator for MListIter {
    type Item = Value;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let item = self.ls.read(|l| l[self.idx].clone());
        self.idx += 1;
        Some(item)
    }
}
