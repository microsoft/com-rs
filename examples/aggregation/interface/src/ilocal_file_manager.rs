use com::{com_interface, interfaces::iunknown::IUnknown};

use winapi::shared::winerror::HRESULT;

#[com_interface("4FC333E3-C389-4C48-B108-7895B0AF21AD")]
pub trait ILocalFileManager: IUnknown {
    unsafe fn delete_local(&self) -> HRESULT;
}
