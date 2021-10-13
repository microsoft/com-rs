use com::interfaces::IUnknown;

com::interfaces! {
    #[uuid("9004239b-61ee-4737-bdc1-f0c2cc42b2e4")]
    pub unsafe interface IFoo : IUnknown {
        fn zap(&self, x: u32);
    }
}

com::class! {
    #[no_class_factory]
    pub class FooServer : IFoo {
        current_x: u32,
        last_words: LastWords,
    }

    impl IFoo for FooServer {
        fn zap(&self, x: u32) {
            eprintln!("FooServer::zap: x = {}", x);
        }
    }
}

pub struct LastWords {
    message: String,
}

impl Drop for LastWords {
    fn drop(&mut self) {
        eprintln!("last words: {}", self.message);
    }
}

/*
impl Drop for FooServer {
    fn drop(&mut self) {
        eprintln!("FooServer::drop");
    }
}
*/

fn main() {
    unsafe {
        println!("Hello, world!");

        let foo_server = FooServer::allocate(
            100,
            LastWords {
                message: "I die...  I die!".to_string(),
            },
        );

        eprintln!("getting IFoo");
        // let foo: IFoo = foo_server.query_interface().unwrap();
        let foo = IFoo::from(&**foo_server);

        eprintln!("dropping server:");
        drop(foo_server);

        eprintln!("calling zap");
        foo.zap(200);

        eprintln!("dropping interface");
        drop(foo);
    }
}
