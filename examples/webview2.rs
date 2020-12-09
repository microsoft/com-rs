use com::interfaces::IUnknown;
use com::sys::{HRESULT, S_OK};
use std::sync::{Arc, Mutex};

com::interfaces! {
    #[uuid("DA66D884-6DA8-410E-9630-8C48F8B3A40E")]
    pub unsafe interface ICoreWebView2Environment : IUnknown {}

    #[uuid("8B4F98CE-DB0D-4E71-85FD-C4C4EF1F2630")]
    pub unsafe interface ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler : IUnknown {
        unsafe fn invoke(
            &self,
            result: HRESULT,
            created_environment: ICoreWebView2Environment
        ) -> HRESULT;
    }
}

com::class! {
    pub class CreateCoreWebView2EnvironmentCompletedHandler : ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler {
        destination: Arc<Mutex<Option<ICoreWebView2Environment>>>,
    }

    impl ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler for CreateCoreWebView2EnvironmentCompletedHandler {
        fn invoke(
            &self,
            result: HRESULT,
            created_environment: ICoreWebView2Environment
        ) -> HRESULT
        {
            if result == S_OK {
                self.destination.lock().unwrap().replace(created_environment);
            }
            result
        }
    }
}

// #[link(name = "WebView2Loader.dll")]
// extern "C" {
//     fn CreateCoreWebView2Environment(
//         environment_created_handler: ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
//     );
// }

// Comment this out and comment the extern symbol in to actually run this
// Note: this requires the WebView2Loader dll and will only work on Windows
#[allow(non_snake_case)]
unsafe fn CreateCoreWebView2Environment(
    _environment_created_handler: ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
) {
    panic!("Called stub function. Uncomment out the extern function from WebViewLoader dll.\n\nNote: this only works on Windows.")
}

pub struct Environment {
    #[allow(unused)]
    raw: ICoreWebView2Environment,
}

impl Environment {
    pub fn create() -> Environment {
        #[cfg(windows)]
        com::runtime::init_apartment(com::runtime::ApartmentType::SingleThreaded)
            .expect("Failed to initialize COM.");

        let environment = Arc::new(Mutex::new(None));

        // create a handler that will store created environments in our local variable
        let handler_class =
            CreateCoreWebView2EnvironmentCompletedHandler::allocate(environment.clone());
        let handler_interface = handler_class
            .query_interface::<ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler>()
            .unwrap();

        // call WebView2 to create an environment and invoke the handler when finished
        unsafe {
            CreateCoreWebView2Environment(handler_interface);
        }
        // Dropping handler class early so that `environment` is the only instance of the `Arc<Mutex<Option<ICoreWebView2Environment>>>`
        drop(handler_class);

        Environment {
            // This *should* work assuming the callback is called synchronously.
            // If this is not the case, then the following will panic.
            raw: Arc::try_unwrap(environment) // Try to unwrap Arc
                .unwrap() // Ensure Arc unwrap worked
                .into_inner() // Unwrap Mutex
                .unwrap() // Ensure mutex unwrap worked
                .unwrap(), // unwrap option
        }
    }
}

fn main() {
    let _environment = Environment::create();
}
