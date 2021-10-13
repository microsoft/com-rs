use com::interfaces::IUnknown;
use std::sync::atomic::{AtomicI32, Ordering::SeqCst};
use std::sync::Arc;

const NUM_DROPPED: i32 = -1;
const NUM_INIT: i32 = 0;

com::interfaces! {
    #[uuid("9004239b-61ee-4737-bdc1-f0c2cc42b2e4")]
    pub unsafe interface IFoo : IUnknown {
        fn zap(&self, i: i32);
    }
}

com::class! {
    #[no_class_factory]
    pub class FooServer : IFoo {
        number: Arc<AtomicI32>,
    }

    impl IFoo for FooServer {
        fn zap(&self, x: i32) {
            println!("FooServer::zap: x = {}", x);
            self.number.store(x, SeqCst);
        }
    }
}

impl Drop for FooServer {
    fn drop(&mut self) {
        println!("FooServer::drop");
        self.number.store(NUM_DROPPED, SeqCst);
    }
}

fn get_refcount<T: com::production::Class>(c: &com::production::ClassAllocation<T>) -> u32 {
    unsafe {
        // barbaric, but effective
        let _ = c.add_ref();
        c.dec_ref_count()
    }
}

fn main() {
    let cell = Arc::new(AtomicI32::new(NUM_INIT));
    let server = FooServer::allocate(cell.clone());
    assert_eq!(get_refcount(&server), 1);
    assert_eq!(cell.load(SeqCst), NUM_INIT);

    let f = IFoo::from(&**server);
    assert_eq!(get_refcount(&server), 2);

    // make a call into server
    println!("calling zap()");
    unsafe {
        f.zap(100);
    }
    assert_eq!(cell.load(SeqCst), 100);

    // clone the ref to the server
    println!("cloning server");
    let server2 = server.clone();
    // verify that cloning the server affected the refcount of the original
    assert_eq!(get_refcount(&server), 3);
    drop(server2);
    assert_eq!(get_refcount(&server), 2);

    // test cloning an interface
    println!("cloning interface");
    let f2 = f.clone();
    assert_eq!(get_refcount(&server), 3);
    drop(f2);
    assert_eq!(get_refcount(&server), 2);

    // drop the server
    println!("dropping server ref");
    drop(server);
    // server.refcount is now 1, but we can't check it any more

    // observe that cell is still alive
    assert_eq!(cell.load(SeqCst), 100);

    // make another server call
    println!("calling zap() again");
    unsafe {
        f.zap(200);
    }
    assert_eq!(cell.load(SeqCst), 200);

    // drop interface
    drop(f);
    assert_eq!(cell.load(SeqCst), NUM_DROPPED);
}
