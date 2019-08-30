use com::Runtime;
use interface::{
    IFileManager, ILocalFileManager, CLSID_LOCAL_FILE_MANAGER_CLASS,
    CLSID_WINDOWS_FILE_MANAGER_CLASS,
};

fn main() {
    let runtime = match Runtime::new() {
        Ok(runtime) => {
            println!("Got a runtime");
            runtime
        }
        Err(hr) => {
            println!("Failed to initialize COM Library: {}", hr);
            return;
        }
    };

    run_aggr_test(runtime);
}

fn run_aggr_test(runtime: Runtime) {
    let result = runtime.create_instance::<dyn IFileManager>(&CLSID_WINDOWS_FILE_MANAGER_CLASS);
    let mut filemanager = match result {
        Ok(filemanager) => filemanager,
        Err(e) => {
            println!("Failed to get filemanager, {:x}", e as u32);
            return;
        }
    };
    println!("Got filemanager!");
    filemanager.delete_all();

    let result = filemanager.get_interface::<dyn ILocalFileManager>();
    let mut lfm = match result {
        Some(lfm) => lfm,
        None => {
            println!("Failed to get Local File Manager!");
            return;
        }
    };
    println!("Got Local File Manager.");
    lfm.delete_local();

    let result = runtime.create_instance::<dyn ILocalFileManager>(&CLSID_LOCAL_FILE_MANAGER_CLASS);
    let mut localfilemanager = match result {
        Ok(localfilemanager) => localfilemanager,
        Err(e) => {
            println!("Failed to get localfilemanager, {:x}", e as u32);
            return;
        }
    };
    println!("Got localfilemanager!");
    localfilemanager.delete_local();
}
