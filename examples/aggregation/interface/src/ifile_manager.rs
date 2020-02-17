use com::{com_interface, interfaces::iunknown::IUnknown, sys::HRESULT};

#[com_interface("25A41124-23D0-46BE-8351-044889D5E37E")]
pub trait IFileManager: IUnknown {
    unsafe fn delete_all(&self) -> HRESULT;
}
