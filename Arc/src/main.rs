use std::{ptr::NonNull, sync::atomic::AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;
struct ArcData<T>{
    ref_count:AtomicUsize,
    data:T,
}
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

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // TODO: Handle overflows.
        self.data().ref_count.fetch_add(1, Relaxed);
        Arc {
            ptr: self.ptr,
        }
    }
}

fn main() {
    println!("Hello, world!");
}
