use com::{interface, IUnknown};

use winapi::shared::winerror::HRESULT;

#[interface(25A41124-23D0-46BE-8351-044889D5E37E)]
pub trait IFileManager: IUnknown {
    fn delete_all(&mut self) -> HRESULT;
}
