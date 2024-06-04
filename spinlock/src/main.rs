use std::{cell::UnsafeCell, sync::atomic::AtomicBool};
use core::sync::atomic::Ordering::{Release,Acquire};
use std::ops::{Deref, DerefMut};
use std::thread;


// if we implement the code without lifetimes then we dont need a Guard but then we have to rely on the user 
// that they will not keep a mutable reference copy returned from the lock


//Guard has no contructor so only way to get it is the lock() method
pub struct Guard<'a,T>{
    spinlock:&'a SpinLock<T>,
}


//need to implement the Deref and Ref traits for convinience and the Drop trait to drop the reference after lock becomes false
impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // The existence of this Guard
        // guarantees we've exclusively locked the lock.
        unsafe { &*self.spinlock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Existence of this Guard
        // guarantees we've exclusively locked the lock.
        unsafe { &mut *self.spinlock.value.get() }
    }
}
impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.spinlock.lock.store(false, Release);
    }
}
// needed to implement to tell compiler that it is safe to share Guard between threads
unsafe impl<T> Send for Guard<'_, T> where T: Send {}
unsafe impl<T> Sync for Guard<'_, T> where T: Sync {}


pub struct SpinLock<T>{
    lock:AtomicBool,
    value: UnsafeCell<T>
}
impl<T> SpinLock<T>{
    pub const fn new(value:T)->Self{ 
        Self{
            lock: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    } 
    pub fn lock(&self)-> Guard<T> { 
        while self.lock.swap(true ,Acquire){
            std::hint::spin_loop(); // might improve processor efficiency
        }
        // unsafe{ &mut *self.value.get()} --> doesnot guarantee dropping of a reference after lifetime ends
        return Guard { spinlock:self };
    }
    pub fn unlock(&self){
        self.lock.store(false, Release);
    }
}
unsafe impl<T> Sync for SpinLock<T> where T: Send {}
fn main() {
    let x = SpinLock::new(Vec::new());
    thread::scope(|s| {
        s.spawn(|| x.lock().push(1));
        s.spawn(|| {
            let mut g = x.lock();
            g.push(2);
            g.push(2);
        });
    });
    let g = x.lock();
    assert!(g.as_slice() == [1, 2, 2] || g.as_slice() == [2, 2, 1]);
}
