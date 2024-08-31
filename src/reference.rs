use dashmap::mapref::one::Ref;
use std::ops::Deref;
use std::sync::Arc;

/// A shared reference to a resource.
pub struct Reference<'a, T> {
    inner: Ref<'a, u64, Arc<T>>,
}

impl<'a, T> Deref for Reference<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> Reference<'a, T> {
    pub fn into_owned(self) -> Arc<T> {
        self.inner.clone()
    }
}

impl<'a, T> From<Ref<'a, u64, Arc<T>>> for Reference<'a, T> {
    fn from(inner: Ref<'a, u64, Arc<T>>) -> Self {
        Self { inner }
    }
}
