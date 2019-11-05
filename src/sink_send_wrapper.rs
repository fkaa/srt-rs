use failure::Error;
use futures::sink::Sink;
/// A wrapper for Sink::Send that allows buffering
use std::collections::VecDeque;
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct SinkSendWrapper<Item> {
    buffer: VecDeque<Item>,
}

impl<Item> Default for SinkSendWrapper<Item> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Item> SinkSendWrapper<Item> {
    pub fn new() -> SinkSendWrapper<Item> {
        SinkSendWrapper {
            buffer: VecDeque::new(),
        }
    }

    pub fn send<S>(&mut self, sink: &mut S, i: Item, cx: &mut Context) -> Result<(), Error>
    where
        S: Sink<Item, Error = Error> + Unpin,
    {
        let mut pin = Pin::new(sink);
        if let Poll::Ready(_) = pin.as_mut().poll_ready(cx) {
            pin.start_send(i)?;
        } else {
            self.buffer.push_back(i);
        }
        Ok(())
    }

    pub fn poll_send<S>(&mut self, sink: &mut S, cx: &mut Context) -> Result<(), Error>
    where
        S: Sink<Item, Error = Error> + Unpin,
    {
        let mut pin = Pin::new(sink);
        while !self.buffer.is_empty() {
            if let Poll::Ready(_) = pin.as_mut().poll_ready(cx) {
                pin.as_mut().start_send(self.buffer.pop_front().unwrap())?
            }
        }
        Ok(())
    }
}
