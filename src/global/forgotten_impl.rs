use std::{any::Any, cell::RefCell, collections::HashMap, mem::ManuallyDrop, rc::Rc};

use super::{ForgottenKey, SharedForgottenKey};

thread_local! {
    static FORGOTTEN: RefCell<Forgotten> =RefCell::new(Forgotten::new());
}

struct Forgotten {
    cur: usize,
    map: HashMap<usize, ManuallyDrop<Rc<dyn Any>>>,
}

impl Forgotten {
    #[inline]
    fn find_available_key(&mut self) -> usize {
        let cur = self.cur;

        loop {
            let (v, _) = self.cur.overflowing_add(1);
            self.cur = v;

            if v == cur {
                panic!("Forgotten is full")
            }

            if v != 0 && !self.map.contains_key(&v) {
                return v;
            }
        }
    }

    #[inline]
    fn insert(&mut self, v: Rc<dyn Any>) -> usize {
        let k = self.find_available_key();
        let v = ManuallyDrop::new(v);

        #[cfg(not(debug_assertions))]
        self.map.insert(k, v);

        #[cfg(debug_assertions)]
        assert!(self.map.insert(k, v).is_none());

        k
    }

    fn new() -> Self {
        Self {
            cur: 0,
            map: HashMap::new(),
        }
    }

    #[inline]
    fn forget<T: Any>(&mut self, v: T) -> ForgottenKey<T> {
        self.forget_rc(Rc::new(v))
    }

    #[inline]
    fn forget_and_get<T: Any>(&mut self, v: T) -> (ForgottenKey<T>, Rc<T>) {
        let v = Rc::new(v);
        let ret = Rc::clone(&v);
        (self.forget_rc(v), ret)
    }

    #[inline]
    fn forget_rc<T: Any>(&mut self, v: Rc<T>) -> ForgottenKey<T> {
        let k = self.insert(v);
        let k = unsafe { ForgottenKey::<T>::new(k) };

        k
    }

    #[inline]
    unsafe fn try_free_with_usize(&mut self, n: usize) -> bool {
        let v = self.map.remove(&n);

        if let Some(mut v) = v {
            ManuallyDrop::drop(&mut v);
            true
        } else {
            false
        }
    }

    #[inline]
    fn free<T: Any>(&mut self, mut k: ForgottenKey<T>) {
        #[cfg(not(debug_assertions))]
        unsafe {
            self.free_with_usize(k.take_usize());
        }

        #[cfg(debug_assertions)]
        assert!(unsafe { self.try_free_with_usize(k.take_usize()) });
    }

    #[inline]
    fn try_free<T: Any>(&mut self, k: &SharedForgottenKey<T>) -> bool {
        unsafe { self.try_free_with_usize(*k.as_usize()) }
    }

    #[inline]
    fn get<T: Any>(&self, k: &ForgottenKey<T>) -> Rc<T> {
        let v = self.map.get(k.as_usize()).unwrap();
        let v = Rc::clone(v);
        let v = v.downcast::<T>().unwrap();
        v
    }

    #[inline]
    fn try_get<T: Any>(&self, k: &SharedForgottenKey<T>) -> Option<Rc<T>> {
        let v = self.map.get(k.as_usize());

        if let Some(v) = v {
            Some(Rc::clone(v).downcast::<T>().unwrap())
        } else {
            None
        }
    }

    #[inline]
    fn take<T: Any>(&mut self, mut k: ForgottenKey<T>) -> Rc<T> {
        let v = self.map.remove(&k.take_usize()).unwrap();
        let v = ManuallyDrop::into_inner(v);
        let v = v.downcast::<T>().unwrap();
        v
    }

    #[inline]
    fn try_take<T: Any>(&mut self, k: &SharedForgottenKey<T>) -> Option<Rc<T>> {
        let v = self.map.remove(k.as_usize());
        if let Some(v) = v {
            Some(ManuallyDrop::into_inner(v).downcast::<T>().unwrap())
        } else {
            None
        }
    }
}

#[inline]
pub fn forget<T: Any>(v: T) -> ForgottenKey<T> {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.forget(v)
    })
}

#[inline]
pub fn forget_and_get<T: Any>(v: T) -> (ForgottenKey<T>, Rc<T>) {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.forget_and_get(v)
    })
}

#[inline]
pub fn forget_rc<T: Any>(v: Rc<T>) -> ForgottenKey<T> {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.forget_rc(v)
    })
}

#[inline]
pub unsafe fn try_free_with_usize(n: usize) -> bool {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.try_free_with_usize(n)
    })
}

#[inline]
pub fn free<T: Any>(k: ForgottenKey<T>) {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.free(k)
    })
}

#[inline]
pub fn try_free<T: Any>(k: &SharedForgottenKey<T>) -> bool {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.try_free(k)
    })
}

#[inline]
pub fn get<T: Any>(k: &ForgottenKey<T>) -> Rc<T> {
    FORGOTTEN.with(|cell| {
        let fg = cell.borrow();
        fg.get(k)
    })
}

#[inline]
pub fn try_get<T: Any>(k: &SharedForgottenKey<T>) -> Option<Rc<T>> {
    FORGOTTEN.with(|cell| {
        let fg = cell.borrow();
        fg.try_get(k)
    })
}

#[inline]
pub fn take<T: Any>(k: ForgottenKey<T>) -> Rc<T> {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.take(k)
    })
}

#[inline]
pub fn try_take<T: Any>(k: &SharedForgottenKey<T>) -> Option<Rc<T>> {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.try_take(k)
    })
}
