use std::{
    any::{Any, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

use super::SharedForgottenKey;

pub struct ForgottenKey<T: Any>(usize, PhantomData<T>);

impl<T: Any> ForgottenKey<T> {
    #[inline]
    pub(super) fn take_usize(&mut self) -> usize {
        std::mem::replace(&mut self.0, 0)
    }

    #[inline]
    pub(super) fn as_usize(&self) -> &usize {
        &self.0
    }

    #[inline]
    pub fn into_shared(mut self) -> SharedForgottenKey<T> {
        SharedForgottenKey::<T>::new(self.take_usize())
    }

    #[inline]
    pub(super) unsafe fn new(n: usize) -> Self {
        Self(n, PhantomData)
    }
}

impl<T: Any> Debug for ForgottenKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(format!("ForgottenKey<{:?}>", TypeId::of::<T>()).as_str())
            .field(&self.0)
            .finish()
    }
}

impl<T: Any> Drop for ForgottenKey<T> {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe { super::try_free_with_usize(self.0) };
        }
    }
}
