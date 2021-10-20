use com::interfaces::iunknown::IUnknown;
use std::sync::{Arc, Mutex};

com::interfaces! {
    #[uuid("00000000-0000-0000-0000-000000000001")]
    pub unsafe interface IFoo: IUnknown {
        fn foo(&self);
    }

    #[uuid("00000000-0000-0000-0000-000000000002")]
    pub unsafe interface IBar: IFoo {
        fn bar(&self);
    }

    #[uuid("00000000-0000-0000-0000-000000000003")]
    pub unsafe interface IZap: IFoo {
        fn zap(&self);
    }
}

com::class! {
    class Server: IBar(IFoo), IZap(IFoo) {
        output: Arc<Mutex<String>>,
    }

    impl IFoo for Server {
        fn foo(&self) {
            self.output.lock().unwrap().push_str("IFoo::foo\n");
        }
    }

    impl IBar for Server {
        fn bar(&self) {
            self.output.lock().unwrap().push_str("IBar::bar\n");
        }
    }

    impl IZap for Server {
        fn zap(&self) {
            self.output.lock().unwrap().push_str("IZap::zap\n");
        }
    }
}

fn main() {
    let output = Arc::new(Mutex::new(String::new()));
    let server = Server::allocate(output.clone());

    let f = IFoo::from(&**server);
    let b = IBar::from(&**server);
    let z = IZap::from(&**server);

    unsafe {
        f.foo();
        b.bar();
        z.zap();
    }

    let output_g = output.lock().unwrap();
    assert_eq!(&*output_g, "IFoo::foo\nIBar::bar\nIZap::zap\n");
}
