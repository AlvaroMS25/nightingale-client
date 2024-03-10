use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use parking_lot::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use crate::events::IncomingEvent;

/// Stream that can be used to receive events from the server. Only one instance can be active at
/// a time.
pub struct EventStream<'a> {
    mutex: &'a Mutex<Option<UnboundedReceiver<IncomingEvent>>>,
    recv: Option<UnboundedReceiver<IncomingEvent>>
}

impl<'a> EventStream<'a> {
    pub(crate) fn new(mutex: &'a Mutex<Option<UnboundedReceiver<IncomingEvent>>>) -> Option<Self> {
        let recv = mutex.lock().take()?;

        Some(Self {
            mutex,
            recv: Some(recv)
        })
    }
}

impl<'a> Stream for EventStream<'a> {
    type Item = IncomingEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().recv.as_mut().unwrap().poll_recv(cx)
    }
}

impl<'a> Drop for EventStream<'a> {
    fn drop(&mut self) {
        *self.mutex.lock() = self.recv.take();
    }
}
