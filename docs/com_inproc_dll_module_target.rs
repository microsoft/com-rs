com_inproc_dll_module![
    (CLSID_CAT_CLASS, BritishShortHairCatClass),
    (CLSID_WINDOWS_FILE_MANAGER_CLASS, WindowsFileManagerClass),
    (CLSID_LOCAL_FILE_MANAGER_CLASS, LocalFileManagerClass)
];

// ------------------------- DESIRED EXPANSION ------------------------------------------------
#[no_mangle]
extern "stdcall" fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {

    unsafe {
        let rclsid_ref = &*rclsid;
        if IsEqualGUID(rclsid_ref, &CLSID_CAT_CLASS) {
            println!("Allocating new object CatClass...");
            let mut cat = Box::new(<BritishShortHairCatClass>::new());
            cat.add_ref();
            let hr = cat.query_interface(riid, ppv);
            cat.release();
            Box::into_raw(cat);

            hr
        } else if IsEqualGUID(rclsid_ref, &CLSID_WINDOWS_FILE_MANAGER_CLASS) {
            println!("Allocating new object WindowsFileManagerClass...");
            let mut wfm = Box::new(<WindowsFileManagerClass>::new());
            wfm.add_ref();
            let hr = wfm.query_interface(riid, ppv);
            wfm.release();
            Box::into_raw(wfm);

            hr
        } else if IsEqualGUID(rclsid_ref, &CLSID_LOCAL_FILE_MANAGER_CLASS) {
            println!("Allocating new object LocalFileManagerClass...");
            let mut lfm = Box::new(<LocalFileManagerClass>::new());
            lfm.add_ref();
            let hr = lfm.query_interface(riid, ppv);
            lfm.release();
            Box::into_raw(lfm);

            hr
        } else {
            CLASS_E_CLASSNOTAVAILABLE
        }
    }
}

// Function tries to add ALL-OR-NONE of the registry keys.
#[no_mangle]
extern "stdcall" fn DllRegisterServer() -> HRESULT {
    let hr = register_keys(get_relevant_registry_keys());
    if failed(hr) {
        DllUnregisterServer();
    }

    hr
}

// Function tries to delete as many registry keys as possible.
#[no_mangle]
extern "stdcall" fn DllUnregisterServer() -> HRESULT {
    let mut registry_keys_to_remove = get_relevant_registry_keys();
    registry_keys_to_remove.reverse();
    unregister_keys(registry_keys_to_remove)
}

fn get_relevant_registry_keys() -> Vec<RegistryKeyInfo> {
    let file_path = get_dll_file_path();
    // IMPORTANT: Assumption of order: Subkeys are located at a higher index than the parent key.
    vec![
        RegistryKeyInfo::new(
            class_key_path(CLSID_CAT_CLASS).as_str(),
            "",
            "BritishShortHairCat",
        ),
        RegistryKeyInfo::new(
            class_inproc_key_path(CLSID_CAT_CLASS).as_str(),
            "",
            file_path.clone().as_str(),
        ),
        RegistryKeyInfo::new(
            class_key_path(CLSID_WINDOWS_FILE_MANAGER_CLASS).as_str(),
            "",
            "WindowsFileManagerClass",
        ),
        RegistryKeyInfo::new(
            class_inproc_key_path(CLSID_WINDOWS_FILE_MANAGER_CLASS).as_str(),
            "",
            file_path.clone().as_str(),
        ),
        RegistryKeyInfo::new(
            class_key_path(CLSID_LOCAL_FILE_MANAGER_CLASS).as_str(),
            "",
            "LocalFileManagerClass",
        ),
        RegistryKeyInfo::new(
            class_inproc_key_path(CLSID_LOCAL_FILE_MANAGER_CLASS).as_str(),
            "",
            file_path.clone().as_str(),
        ),
        
    ]
}
