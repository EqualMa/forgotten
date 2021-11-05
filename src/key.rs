use std::{
    any::{Any, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

#[derive(Hash)]
pub struct ForgottenKey<T: Any>(u32, PhantomData<T>);

impl<T: Any> Debug for ForgottenKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(format!("ForgottenKey<{:?}>", TypeId::of::<T>()).as_str())
            .field(&self.0)
            .finish()
    }
}

impl<T: Any> Drop for ForgottenKey<T> {
    fn drop(&mut self) {
        todo!()
    }
}

#[derive(Hash)]
pub struct SharedForgottenKey<T: Any>(u32, PhantomData<T>);

impl<T: Any> PartialEq for SharedForgottenKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Any> Eq for SharedForgottenKey<T> {}

impl<T: Any> Clone for SharedForgottenKey<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T: Any> Copy for SharedForgottenKey<T> {}

impl<T: Any> Debug for SharedForgottenKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(format!("SharedForgottenKey<{:?}>", TypeId::of::<T>()).as_str())
            .field(&self.0)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnyForgottenKey(u32);

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
