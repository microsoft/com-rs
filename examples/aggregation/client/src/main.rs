use com::runtime::ApartmentThreadedRuntime as Runtime;
use interface::{
    IFileManager, ILocalFileManager, CLSID_LOCAL_FILE_MANAGER_CLASS,
    CLSID_WINDOWS_FILE_MANAGER_CLASS,
};

fn main() {
    let runtime =
        Runtime::new().unwrap_or_else(|hr| panic!("Failed to initialize COM Library{:x}", hr));
    println!("Got a runtime");

    let file_manager = runtime
        .create_instance::<dyn IFileManager>(&CLSID_WINDOWS_FILE_MANAGER_CLASS)
        .unwrap_or_else(|hr| panic!("Failed to get file manager{:x}", hr));
    println!("Got filemanager!");
    file_manager.delete_all();

    let local_file_manager = file_manager
        .get_interface::<dyn ILocalFileManager>()
        .expect("Failed to get local file manager");
    println!("Got local lile lanager.");
    local_file_manager.delete_local();

    let local_file_manager = runtime
        .create_instance::<dyn ILocalFileManager>(&CLSID_LOCAL_FILE_MANAGER_CLASS)
        .unwrap_or_else(|hr| panic!("Failed to get local file manager{:x}", hr));
    println!("Got localfilemanager!");
    local_file_manager.delete_local();
}
