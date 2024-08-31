use std::ops::{Deref, DerefMut};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::sync::{
    RwLock as TokioRwLock,
    RwLockReadGuard as TokioRwLockReadGuard,
    RwLockWriteGuard as TokioRwLockWriteGuard,
};

pub struct RefLock<T> {
    inner: RwLock<T>
}

impl<T> RefLock<T> {
    pub fn new(value: T) -> RefLock<T> {
        Self {
            inner: RwLock::new(value)
        }
    }

    pub fn read(&self) -> Read<T> {
        Read {
            inner: self.inner.read()
        }
    }

    pub fn write(&self) -> Write<T> {
        Write {
            inner: self.inner.write()
        }
    }
}

pub struct Read<'a, T> {
    inner: RwLockReadGuard<'a, T>
}

impl<'a, T> Deref for Read<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

pub struct Write<'a, T> {
    inner: RwLockWriteGuard<'a, T>
}

impl<'a, T> Deref for Write<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<'a, T> DerefMut for Write<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: Default> Default for RefLock<T> {
    fn default() -> RefLock<T> {
        RefLock::new(Default::default())
    }
}

#[allow(dead_code)]
pub struct AsyncRefLock<T> {
    inner: TokioRwLock<T>
}

#[allow(dead_code)]
impl<T> AsyncRefLock<T> {
    pub fn new(value: T) -> AsyncRefLock<T> {
        Self {
            inner: TokioRwLock::new(value)
        }
    }

    pub async fn read(&self) -> AsyncRead<T> {
        AsyncRead {
            inner: self.inner.read().await
        }
    }

    pub async fn write(&self) -> AsyncWrite<T> {
        AsyncWrite {
            inner: self.inner.write().await
        }
    }
}

pub struct AsyncRead<'a, T> {
    inner: TokioRwLockReadGuard<'a, T>
}

impl<'a, T> Deref for AsyncRead<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

pub struct AsyncWrite<'a, T> {
    inner: TokioRwLockWriteGuard<'a, T>
}

impl<'a, T> Deref for AsyncWrite<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<'a, T> DerefMut for AsyncWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: Default> Default for AsyncRefLock<T> {
    fn default() -> AsyncRefLock<T> {
        AsyncRefLock::new(Default::default())
    }
}
