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
    fn forget_rc<T: ?Sized + Any>(&mut self, v: Rc<T>) -> ForgottenKey<T> {
        let k = self.insert(v as Rc<dyn Any>);
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
    fn free<T: ?Sized + Any>(&mut self, mut k: ForgottenKey<T>) {
        #[cfg(not(debug_assertions))]
        unsafe {
            self.free_with_usize(k.take_usize());
        }

        #[cfg(debug_assertions)]
        assert!(unsafe { self.try_free_with_usize(k.take_usize()) });
    }

    #[inline]
    fn try_free<T: ?Sized + Any>(&mut self, k: &SharedForgottenKey<T>) -> bool {
        unsafe { self.try_free_with_usize(*k.as_usize()) }
    }

    #[inline]
    fn get<T: ?Sized + Any>(&self, k: &ForgottenKey<T>) -> Rc<T> {
        let v = self.map.get(k.as_usize()).unwrap();
        let v = Rc::clone(v);
        let v = v.downcast::<T>().unwrap();
        v
    }

    #[inline]
    fn try_get<T: ?Sized + Any>(&self, k: &SharedForgottenKey<T>) -> Option<Rc<T>> {
        let v = self.map.get(k.as_usize());

        if let Some(v) = v {
            Some(Rc::clone(v).downcast::<T>().unwrap())
        } else {
            None
        }
    }

    #[inline]
    fn take<T: ?Sized + Any>(&mut self, mut k: ForgottenKey<T>) -> Rc<T> {
        let v = self.map.remove(&k.take_usize()).unwrap();
        let v = ManuallyDrop::into_inner(v);
        let v = v.downcast::<T>().unwrap();
        v
    }

    #[inline]
    fn try_take<T: ?Sized + Any>(&mut self, k: &SharedForgottenKey<T>) -> Option<Rc<T>> {
        let v = self.map.remove(k.as_usize());
        if let Some(v) = v {
            Some(ManuallyDrop::into_inner(v).downcast::<T>().unwrap())
        } else {
            None
        }
    }
}

#[inline]
pub fn forget<T: ?Sized + Any>(v: T) -> ForgottenKey<T> {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.forget(v)
    })
}

#[inline]
pub fn forget_and_get<T: ?Sized + Any>(v: T) -> (ForgottenKey<T>, Rc<T>) {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.forget_and_get(v)
    })
}

#[inline]
pub fn forget_rc<T: ?Sized + Any>(v: Rc<T>) -> ForgottenKey<T> {
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
pub fn free<T: ?Sized + Any>(k: ForgottenKey<T>) {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.free(k)
    })
}

#[inline]
pub fn try_free<T: ?Sized + Any>(k: &SharedForgottenKey<T>) -> bool {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.try_free(k)
    })
}

#[inline]
pub fn get<T: ?Sized + Any>(k: &ForgottenKey<T>) -> Rc<T> {
    FORGOTTEN.with(|cell| {
        let fg = cell.borrow();
        fg.get(k)
    })
}

#[inline]
pub fn try_get<T: ?Sized + Any>(k: &SharedForgottenKey<T>) -> Option<Rc<T>> {
    FORGOTTEN.with(|cell| {
        let fg = cell.borrow();
        fg.try_get(k)
    })
}

#[inline]
pub fn take<T: ?Sized + Any>(k: ForgottenKey<T>) -> Rc<T> {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.take(k)
    })
}

#[inline]
pub fn try_take<T: ?Sized + Any>(k: &SharedForgottenKey<T>) -> Option<Rc<T>> {
    FORGOTTEN.with(|cell| {
        let mut fg = cell.borrow_mut();
        fg.try_take(k)
    })
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    #[test]
    fn test_drop_key_1() {
        thread_local! {
            static DROPPED: RefCell<Vec<String>> = RefCell::new(vec![]);
        }

        struct MyValue {
            v: String,
        }

        impl Drop for MyValue {
            fn drop(&mut self) {
                let v = self.v.clone();
                DROPPED.with(|cell| {
                    cell.borrow_mut().push(v);
                })
            }
        }

        let v = MyValue {
            v: "some value".to_string(),
        };

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (0, 0)
        );

        let k = super::forget(v);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 1)
        );

        assert_eq!(*k.as_usize(), 1);

        let rc = super::get(&k);

        assert_eq!(rc.v, "some value");

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 1)
        );

        drop(k);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 0)
        );

        DROPPED.with(|cell| {
            assert_eq!(cell.borrow().len(), 0);
        });

        drop(rc);

        DROPPED.with(|cell| {
            assert_eq!(*cell.borrow(), ["some value"]);
        });
    }

    #[test]
    fn test_drop_key_1_1() {
        test_drop_key_1();

        thread_local! {
            static DROPPED: RefCell<Vec<String>> = RefCell::new(vec![]);
        }

        struct MyValue {
            v: String,
        }

        impl Drop for MyValue {
            fn drop(&mut self) {
                let v = self.v.clone();
                DROPPED.with(|cell| {
                    cell.borrow_mut().push(v);
                })
            }
        }

        let v = MyValue {
            v: "some value 2".to_string(),
        };

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 0)
        );

        let k = super::forget(v);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (2, 1)
        );

        assert_eq!(*k.as_usize(), 2);

        let rc = super::get(&k);

        assert_eq!(rc.v, "some value 2");

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (2, 1)
        );

        drop(k);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (2, 0)
        );

        DROPPED.with(|cell| {
            assert_eq!(cell.borrow().len(), 0);
        });

        drop(rc);

        DROPPED.with(|cell| {
            assert_eq!(*cell.borrow(), ["some value 2"]);
        });
    }

    #[test]
    fn test_drop_key_2() {
        thread_local! {
            static DROPPED: RefCell<Vec<String>> = RefCell::new(vec![]);
        }

        struct MyValue {
            v: String,
        }

        impl Drop for MyValue {
            fn drop(&mut self) {
                let v = self.v.clone();
                DROPPED.with(|cell| {
                    cell.borrow_mut().push(v);
                })
            }
        }

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (0, 0)
        );

        let v = MyValue {
            v: "some value".to_string(),
        };

        let k = super::forget(v);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 1)
        );

        assert_eq!(*k.as_usize(), 1);

        let rc = super::get(&k);

        assert_eq!(rc.v, "some value");

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 1)
        );

        let v2 = MyValue {
            v: "some value 2".to_string(),
        };

        let k2 = super::forget(v2);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (2, 2)
        );

        assert_eq!(*k2.as_usize(), 2);

        let rc2 = super::get(&k2);

        assert_eq!(rc2.v, "some value 2");

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (2, 2)
        );

        drop(k);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (2, 1)
        );

        {
            assert_eq!(*k2.as_usize(), 2);

            let rc2 = super::get(&k2);

            assert_eq!(rc2.v, "some value 2");

            assert_eq!(
                super::FORGOTTEN.with(|cell| {
                    let fg = cell.borrow();
                    (fg.cur, fg.map.len())
                }),
                (2, 1)
            );
        }

        drop(k2);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (2, 0)
        );

        DROPPED.with(|cell| {
            assert_eq!(cell.borrow().len(), 0);
        });

        drop(rc2);

        DROPPED.with(|cell| {
            assert_eq!(*cell.borrow(), ["some value 2"]);
        });

        drop(rc);

        DROPPED.with(|cell| {
            assert_eq!(*cell.borrow(), ["some value 2", "some value"]);
        });
    }

    #[test]
    fn test_shared_key() {
        thread_local! {
            static DROPPED: RefCell<Vec<String>> = RefCell::new(vec![]);
        }

        struct MyValue {
            v: String,
        }

        impl Drop for MyValue {
            fn drop(&mut self) {
                let v = self.v.clone();
                DROPPED.with(|cell| {
                    cell.borrow_mut().push(v);
                })
            }
        }

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (0, 0)
        );

        let v = MyValue {
            v: "some value".to_string(),
        };

        let k = super::forget(v);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 1)
        );

        assert_eq!(*k.as_usize(), 1);

        let k = k.into_shared();

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 1)
        );

        {
            let rc = super::try_get(&k);

            assert!(if let Some(v) = rc {
                v.v == "some value"
            } else {
                false
            });
        }

        let k1 = k.clone();

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 1)
        );

        drop(k);

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 1)
        );

        {
            let rc1 = super::try_get(&k1);

            DROPPED.with(|cell| {
                assert_eq!(cell.borrow().len(), 0);
            });

            assert!(if let Some(v) = &rc1 {
                v.v == "some value"
            } else {
                false
            });

            DROPPED.with(|cell| {
                assert_eq!(cell.borrow().len(), 0);
            });

            assert!(super::try_free(&k1));

            assert_eq!(
                super::FORGOTTEN.with(|cell| {
                    let fg = cell.borrow();
                    (fg.cur, fg.map.len())
                }),
                (1, 0)
            );

            assert!(super::try_get(&k1).is_none());
            assert!(!super::try_free(&k1));

            assert_eq!(
                super::FORGOTTEN.with(|cell| {
                    let fg = cell.borrow();
                    (fg.cur, fg.map.len())
                }),
                (1, 0)
            );

            DROPPED.with(|cell| {
                assert_eq!(cell.borrow().len(), 0);
            });
        }

        DROPPED.with(|cell| {
            assert_eq!(*cell.borrow(), ["some value"]);
        });
    }

    #[test]
    fn test_dropped_shared_key() {
        thread_local! {
            static DROPPED: RefCell<Vec<String>> = RefCell::new(vec![]);
        }

        struct MyValue {
            v: String,
        }

        impl Drop for MyValue {
            fn drop(&mut self) {
                let v = self.v.clone();
                DROPPED.with(|cell| {
                    cell.borrow_mut().push(v);
                })
            }
        }

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (0, 0)
        );

        {
            let v = MyValue {
                v: "some value".to_string(),
            };

            let k = super::forget(v);

            assert_eq!(
                super::FORGOTTEN.with(|cell| {
                    let fg = cell.borrow();
                    (fg.cur, fg.map.len())
                }),
                (1, 1)
            );

            assert_eq!(*k.as_usize(), 1);

            k.into_shared();
        }

        DROPPED.with(|cell| {
            assert_eq!(cell.borrow().len(), 0);
        });

        assert_eq!(
            super::FORGOTTEN.with(|cell| {
                let fg = cell.borrow();
                (fg.cur, fg.map.len())
            }),
            (1, 1)
        );

        {
            let k1 = unsafe { crate::SharedForgottenKey::<MyValue>::from_usize(1) };
            let rc1 = super::try_get(&k1);

            assert!(if let Some(v) = &rc1 {
                v.v == "some value"
            } else {
                false
            });

            assert_eq!(
                super::FORGOTTEN.with(|cell| {
                    let fg = cell.borrow();
                    (fg.cur, fg.map.len())
                }),
                (1, 1)
            );
            DROPPED.with(|cell| {
                assert_eq!(cell.borrow().len(), 0);
            });

            assert!(super::try_free(&k1));

            assert_eq!(
                super::FORGOTTEN.with(|cell| {
                    let fg = cell.borrow();
                    (fg.cur, fg.map.len())
                }),
                (1, 0)
            );

            assert!(super::try_get(&k1).is_none());
            assert!(!super::try_free(&k1));

            assert_eq!(
                super::FORGOTTEN.with(|cell| {
                    let fg = cell.borrow();
                    (fg.cur, fg.map.len())
                }),
                (1, 0)
            );

            DROPPED.with(|cell| {
                assert_eq!(cell.borrow().len(), 0);
            });
        }

        DROPPED.with(|cell| {
            assert_eq!(*cell.borrow(), ["some value"]);
        });
    }
}
