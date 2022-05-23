use std::{cell::RefCell, collections::HashMap, rc::Rc};

use num::{traits::ops::overflowing::OverflowingAdd, One, Zero};

#[derive(Debug)]
pub struct Forgotten<K, T: ?Sized> {
    cur: K,
    map: HashMap<K, Rc<T>>,
}

impl<K: Clone + Eq + std::hash::Hash + OverflowingAdd + Zero + One, T: ?Sized> Forgotten<K, T> {
    #[inline]
    fn find_available_key(&mut self) -> Option<K> {
        let mut k = self.cur.clone();

        loop {
            (k, _) = k.overflowing_add(&K::one());

            if k == self.cur {
                return None;
            }

            if !k.is_zero() && !self.map.contains_key(&k) {
                self.cur = k.clone();
                return Some(k);
            }
        }
    }

    #[inline]
    fn insert(&mut self, v: Rc<T>) -> K {
        let k = self.find_available_key().expect("Forgotten is full");

        #[cfg(not(debug_assertions))]
        self.map.insert(k, v);

        #[cfg(debug_assertions)]
        assert!(self.map.insert(k.clone(), v).is_none());

        k
    }

    pub fn new() -> Self {
        Self {
            cur: K::zero(),
            map: HashMap::new(),
        }
    }

    #[inline]
    pub fn forget(&mut self, v: T) -> K
    where
        T: Sized,
    {
        self.forget_rc(Rc::new(v))
    }

    #[inline]
    pub fn forget_and_get(&mut self, v: T) -> (K, Rc<T>)
    where
        T: Sized,
    {
        let v = Rc::new(v);
        let ret = Rc::clone(&v);
        (self.forget_rc(v), ret)
    }

    #[inline]
    pub fn forget_rc(&mut self, v: Rc<T>) -> K {
        let k = self.insert(v);
        k
    }

    #[inline]
    pub fn try_free(&mut self, k: &K) -> bool {
        let v = self.map.remove(k);

        v.is_some()
    }

    #[inline]
    pub fn try_get(&self, k: &K) -> Option<Rc<T>> {
        self.try_ref(k).map(Rc::clone)
    }

    #[inline]
    pub fn try_ref(&self, k: &K) -> Option<&Rc<T>> {
        self.map.get(k)
    }

    #[inline]
    pub fn try_take(&mut self, k: &K) -> Option<Rc<T>> {
        let v = self.map.remove(k);
        if let Some(v) = v {
            Some(v)
        } else {
            None
        }
    }
}

pub struct ForgottenRefCell<K, T: ?Sized>(std::cell::RefCell<Forgotten<K, T>>);

impl<K: Clone + Eq + std::hash::Hash + OverflowingAdd + Zero + One, T: ?Sized>
    ForgottenRefCell<K, T>
{
    pub fn new() -> Self {
        Self(RefCell::new(Forgotten::new()))
    }

    #[inline]
    pub fn forget(&self, v: T) -> K
    where
        T: Sized,
    {
        self.0.borrow_mut().forget(v)
    }

    #[inline]
    pub fn forget_and_get(&self, v: T) -> (K, Rc<T>)
    where
        T: Sized,
    {
        self.0.borrow_mut().forget_and_get(v)
    }

    #[inline]
    pub fn forget_rc(&self, v: Rc<T>) -> K {
        self.0.borrow_mut().forget_rc(v)
    }

    #[inline]
    pub fn try_free(&self, k: &K) -> bool {
        self.0.borrow_mut().try_free(k)
    }

    #[inline]
    pub fn try_get(&self, k: &K) -> Option<Rc<T>> {
        self.0.borrow().try_get(k)
    }

    #[inline]
    pub fn try_take(&self, k: &K) -> Option<Rc<T>> {
        self.0.borrow_mut().try_take(k)
    }
}
