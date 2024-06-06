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



#[test]
fn test() {
    static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

    struct DetectDrop;

    impl Drop for DetectDrop {
        fn drop(&mut self) {
            NUM_DROPS.fetch_add(1, Relaxed);
        }
    }

    // Create two Arcs sharing an object containing a string
    // and a DetectDrop, to detect when it's dropped.
    let x = Arc::new(("hello", DetectDrop));
    let y = x.clone();

    // Send x to another thread, and use it there.
    let t = std::thread::spawn(move || {
        assert_eq!(x.0, "hello");
    });

    // In parallel, y should still be usable here.
    assert_eq!(y.0, "hello");

    // Wait for the thread to finish.
    t.join().unwrap();

    // One Arc, x, should be dropped by now.
    // We still have y, so the object shouldn't have been dropped yet.
    assert_eq!(NUM_DROPS.load(Relaxed), 0);

    // Drop the remaining `Arc`.
    drop(y);

    // Now that `y` is dropped too,
    // the object should've been dropped.
    assert_eq!(NUM_DROPS.load(Relaxed), 1);
}
