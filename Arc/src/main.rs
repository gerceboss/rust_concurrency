use std::{ptr::NonNull, sync::atomic::AtomicUsize};
use std::sync::atomic::Ordering::{Release,Acquire,Relaxed};
use std::ops::Deref;
use std::sync::atomic::fence;
struct ArcData<T>{
    ref_count:AtomicUsize,
    data:T,
}
// we did not use dorectly Box allocation because we need shared ownershp and not exclusive one
struct Arc<T>{
    ptr:NonNull<ArcData<T>>,
}

unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}


/*Box::new to create a new allocation, 
Box::leak to give up our exclusive ownership of this allocation, 
NonNull::from to turn it into a pointer*/ 
impl <T> Arc<T>{
    pub fn new(data:T)->Arc<T>{
        Arc{
            ptr:NonNull::from(Box::leak(Box::new(ArcData{
                ref_count:AtomicUsize::new(1),
                data:data,
                }))),
        }
    }
    fn data(&self)->&ArcData<T> {
        unsafe{self.ptr.as_ref()}
    }
}


impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data().data
    }
}


impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        // we need to make sure that before dropping no one is accessing that data so we reduce the ref_count
        if self.data().ref_count.fetch_sub(1,Release) == 1 {
            fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}
impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        
        // handles overflows
        if self.data().ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        self.data().ref_count.fetch_add(1, Relaxed);
        Arc {
            ptr: self.ptr,
        }
    }
}

fn main() {
    println!("Hello, world!");
}
