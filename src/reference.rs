use std::ops::{Deref, DerefMut};
use dashmap::mapref::one::{Ref, RefMut};

/// A shared reference to a resource.
pub struct Reference<'a, T>(Ref<'a, u64, T>);

impl<'a, T> Deref for Reference<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.value()
    }
}

/// An exclusive reference to a resource.
pub struct ReferenceMut<'a, T>(RefMut<'a, u64, T>);

impl<'a, T> Deref for ReferenceMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.value()
    }
}

impl<'a, T> DerefMut for ReferenceMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.value_mut()
    }
}

impl<'a, T> From<Ref<'a, u64, T>> for Reference<'a, T> {
    fn from(value: Ref<'a, u64, T>) -> Self {
        Self(value)
    }
}

impl<'a, T> From<RefMut<'a, u64, T>> for ReferenceMut<'a, T> {
    fn from(value: RefMut<'a, u64, T>) -> Self {
        Self(value)
    }
}
