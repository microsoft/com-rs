use com::{
    create_instance, failed, initialize_ex, uninitialize, ComInterface, ComPtr, IClassFactory,
    IUnknown, IID_ICLASSFACTORY,
};
use interface::{
    IFileManager, ILocalFileManager, CLSID_LOCAL_FILE_MANAGER_CLASS,
    CLSID_WINDOWS_FILE_MANAGER_CLASS,
};

fn main() {
    let result = initialize_ex();

    if let Err(hr) = result {
        println!("Failed to initialize COM Library: {}", hr);
        return;
    }

    run_aggr_test();

    uninitialize();
}

fn run_aggr_test() {
    let result = create_instance::<IFileManager>(&CLSID_WINDOWS_FILE_MANAGER_CLASS);
    let mut filemanager = match result {
        Ok(filemanager) => filemanager,
        Err(e) => {
            println!("Failed to get filemanager, {:x}", e as u32);
            return;
        }
    };
    println!("Got filemanager!");
    filemanager.delete_all();

    let result = filemanager.get_interface::<ILocalFileManager>();
    let mut lfm = match result {
        Some(lfm) => lfm,
        None => {
            println!("Failed to get Local File Manager!");
            return;
        }
    };
    println!("Got Local File Manager.");
    lfm.delete_local();

    let result = create_instance::<ILocalFileManager>(&CLSID_LOCAL_FILE_MANAGER_CLASS);
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
