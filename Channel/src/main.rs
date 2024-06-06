use std::thread::Thread;
use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::mem::MaybeUninit; // unsafe Option of rust
use std::sync::atomic::Ordering::{Acquire,Release};
use std::marker::PhantomData;
use std::thread;

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}
pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
    receiving_thread: Thread, 
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
    _no_send: PhantomData<*const ()>, 
}

impl<T> Sender<'_, T> {
    pub fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel.ready.store(true, Release);
        self.receiving_thread.unpark(); // so that user doesnot need ti handle it 
    }
}

impl<T> Receiver<'_, T> {
    pub fn receive(self) -> T {
        while !self.channel.ready.swap(false, Acquire) {
            thread::park();
        }
        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }

// for getting the sender and receiver separately from the one-shot channel

    pub fn split<'a>(&'a mut self) -> (Sender<'a, T>, Receiver<'a, T>) {
        *self = Self::new();
        (
            Sender {    
                channel: self,
                receiving_thread: thread::current(), // 
            },
            Receiver {
                channel: self,
                _no_send: PhantomData,
            }
        )
    }
}
// so that the borrowed value will be dropped after lock is released
impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe { self.message.get_mut().assume_init_drop() }
        }
    }
}

fn main() {
    let mut channel = Channel::new();
    thread::scope(|s| {
        let (sender, receiver) = channel.split();
        s.spawn(move || {
            sender.send("hello world!");
        });
        assert_eq!(receiver.receive(), "hello world!");
    });
}