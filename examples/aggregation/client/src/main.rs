use com::runtime::{create_instance, init_runtime};
use interface::{
    IFileManager, ILocalFileManager, CLSID_LOCAL_FILE_MANAGER_CLASS,
    CLSID_WINDOWS_FILE_MANAGER_CLASS,
};

fn main() {
    init_runtime().unwrap_or_else(|hr| panic!("Failed to initialize COM Library{:x}", hr));
    println!("Got a runtime");

    let file_manager = create_instance::<dyn IFileManager>(&CLSID_WINDOWS_FILE_MANAGER_CLASS)
        .unwrap_or_else(|hr| panic!("Failed to get file manager{:x}", hr));
    println!("Got filemanager!");
    unsafe { file_manager.delete_all() };

    let local_file_manager = file_manager
        .get_interface::<dyn ILocalFileManager>()
        .expect("Failed to get local file manager");
    println!("Got local lile lanager.");
    unsafe { local_file_manager.delete_local() };

    let local_file_manager =
        create_instance::<dyn ILocalFileManager>(&CLSID_LOCAL_FILE_MANAGER_CLASS)
            .unwrap_or_else(|hr| panic!("Failed to get local file manager{:x}", hr));
    println!("Got localfilemanager!");
    unsafe { local_file_manager.delete_local() };
}
