#![cfg(feature = "custom")]

use forgotten::{Forgotten, ForgottenRefCell};
use std::{fmt::Display, ops::Deref, rc::Rc};

#[test]
fn use_custom_forgotten() {
    let mut f = Forgotten::<u8, dyn Display>::new();
    {
        let k = f.forget_rc(Rc::new(0));
        assert_eq!(k, 1);
        let v = f.try_take(&k).unwrap();
        assert_eq!(v.to_string(), "0");
    }

    {
        let k = f.forget_rc(Rc::new("hello world!"));
        assert_eq!(k, 2);
        assert_eq!(f.try_get(&k).unwrap().to_string(), "hello world!");
    }

    for i in 3..=(u8::MAX) {
        let k = f.forget_rc(Rc::new(i));
        assert_eq!(k, i);
        assert_eq!(f.try_get(&k).unwrap().to_string(), k.to_string());
    }

    {
        let k = f.forget_rc(Rc::new(1.1));
        assert_eq!(k, 1);
        let v = f.try_get(&k).unwrap();
        assert_eq!(v.to_string(), "1.1");
    }

    assert_eq!(f.try_get(&2).unwrap().to_string(), "hello world!");
    assert_eq!(f.try_get(&3).unwrap().to_string(), "3");

    assert_eq!(f.try_get(&u8::MAX).unwrap().to_string(), "255");
}

#[test]
fn use_custom_forgotten_ref_cell() {
    thread_local! {
        static F: ForgottenRefCell<i8, i32> = ForgottenRefCell::new()
    }

    for i in 0..i8::MAX {
        let v: i32 = i.into();
        let k = F.with(|f| f.forget(v));

        assert_eq!(k, i + 1);
        assert_eq!(F.with(|f| f.try_take(&k)).unwrap().deref(), &v);
    }

    for i in i8::MIN..0 {
        let v: i32 = i.into();
        let k = F.with(|f| f.forget(v));

        assert_eq!(k, i);
        assert_eq!(F.with(|f| f.try_get(&k)).unwrap().deref(), &v);
    }

    {
        let k = F.with(|f| f.forget(0));
        assert_eq!(k, 1);
        assert_eq!(F.with(|f| f.try_get(&k)).unwrap().deref(), &0);
    }
}
