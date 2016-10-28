#![allow(dead_code)]
extern crate gj;
extern crate gjio;
use std::sync::mpsc::Receiver;
use std::io::{Cursor, Error, Read};

struct ReadFulfillment <T> where T: 'static {
    fulfiller: ::gj::PromiseFulfiller<(T, usize), Error>,
    min_bytes: usize,
    buf: T
}

pub struct ChannelReader<T> where T: 'static {
    port: Receiver<Vec<u8>>,
    fulfillments: Vec<ReadFulfillment<T>>,
    buf: Vec<u8>
}

impl<T> ChannelReader<T> where T: AsMut<[u8]> {
    pub fn new(port: Receiver<Vec<u8>>) -> ChannelReader<T> {
        ChannelReader {
            port: port,
            fulfillments: vec!(),
            buf: vec!()
        }
    }

    pub fn start(&mut self) {
        for msg in self.port.iter() {
            self.buf.extend(msg);
            let mut promise = self.fulfillments.pop().unwrap(); 
            let read_bytes = Cursor::new(&mut self.buf).read(promise.buf.as_mut()).unwrap();
            promise.fulfiller.fulfill((promise.buf, read_bytes));
        }
    }
}

// See: https://docs.capnproto-rust.org/gjio/trait.AsyncRead.html
impl<T> ::gjio::AsyncRead for ChannelReader<T> where T: AsMut<[u8]> {
    fn try_read<T>(&mut self, buf: T, min_bytes: usize) -> ::gj::Promise<(T, usize), Error>
        where T: AsMut<[u8]>
    {
        let (promise, fulfiller) = ::gj::Promise::and_fulfiller();
        self.fulfillments.push(ReadFulfillment {
            fulfiller: fulfiller,
            min_bytes: min_bytes,
            buf: buf
        });
        return promise;
    }
}
