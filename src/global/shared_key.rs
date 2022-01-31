use std::{
    any::{Any, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

#[derive(Hash)]
pub struct SharedForgottenKey<T: ?Sized + Any>(usize, PhantomData<T>);

impl<T: ?Sized + Any> SharedForgottenKey<T> {
    pub(crate) fn new(n: usize) -> Self {
        Self(n, PhantomData)
    }
}

impl<T: ?Sized + Any> SharedForgottenKey<T> {
    pub fn as_usize(&self) -> &usize {
        &self.0
    }

    pub fn into_type_and_usize(self) -> (TypeId, usize) {
        (TypeId::of::<T>(), self.0)
    }

    pub unsafe fn from_usize(n: usize) -> Self {
        Self(n, PhantomData)
    }
}

impl<T: ?Sized + Any> PartialEq for SharedForgottenKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized + Any> Eq for SharedForgottenKey<T> {}

impl<T: ?Sized + Any> Clone for SharedForgottenKey<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T: ?Sized + Any> Copy for SharedForgottenKey<T> {}

impl<T: ?Sized + Any> Debug for SharedForgottenKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(format!("SharedForgottenKey<{:?}>", TypeId::of::<T>()).as_str())
            .field(&self.0)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::SharedForgottenKey;

    #[test]
    fn test_clone_eq() {
        struct NotClone {
            _val: u8,
        }

        let a = SharedForgottenKey::<NotClone>(1, PhantomData);

        let b = a.clone();

        assert_eq!(a, b);
    }
}
