use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::value::Value;

// stands for MothReference
#[derive(Debug, Clone)]
pub struct MRef<T>(Rc<UnsafeCell<T>>);
impl<T> MRef<T> {
    pub fn new(val: T) -> Self {
        MRef(Rc::new(UnsafeCell::new(val)))
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
        MRef::new(value)
    }
}

impl<T: PartialEq> PartialEq for MRef<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.0.get() == *other.0.get() }
    }
}


pub type MMap<K, T> = MRef<HashMap<K, T>>;
impl<K: Eq + PartialEq + Hash + Clone + 'static, T: Clone> MMap<K, T> {
    pub fn insert(&mut self, key: K, val: T) {
        unsafe {
            let map = &mut *self.0.get();
            map.insert(key, val);
        }
    }

    pub fn get(&self, key: &K) -> Option<&T> {
        unsafe {
            let map = &*self.0.get();
            map.get(key)
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (K, T)> {
        MMapIter::new(self.clone())
    }
}
// /*
struct MMapIter<K: Eq + PartialEq + Hash, T> {
    idx: usize,
    len: usize,
    keys: Vec<K>,
    map: MMap<K, T>,
}
impl<K: Eq + PartialEq + Hash + Clone + 'static, T> MMapIter<K, T> {
    pub fn new(map: MMap<K, T>) -> Self {
        MMapIter {
            idx: 0,
            len: map.read(|m| m.len()),
            keys: map.read(|m| m.keys().cloned().collect::<Vec<_>>()),
            map,
        }
    }
}
impl<K: Eq + PartialEq + Hash + Clone + 'static, T: Clone> Iterator for MMapIter<K, T> {
    type Item = (K, T);
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
// */

pub type MList = MRef<Vec<Value>>;
impl MList {
    pub fn modify(&mut self, idx: usize, val: Value) {
        unsafe {
            let ls = &mut *self.0.get();
            ls[idx] = val;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Value> {
        MListIter::new(self.clone())
    }

    // checks whether it is in the possible range (even if negative)
    // and returns it as a positive index
    pub fn check_index(idx: i32, length: usize) -> Option<usize> {
        if length as i32 <= idx || idx < -(length as i32) {
            return None;
        }
        Some(if idx < 0 { length as i32 + idx } else { idx } as usize)
    }
}
struct MListIter {
    idx: usize,
    len: usize,
    ls: MList,
}
impl MListIter {
    pub fn new(ls: MList) -> Self {
        let len = ls.read(|l| l.len());
        MListIter { ls, len, idx: 0 }
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
