fn main() {
    windows::build! {
        Windows::Win32::Foundation::{BOOLEAN, HANDLE, HWND, PSTR, PWSTR},
        Windows::Win32::System::Diagnostics::Debug::{SetLastError, WIN32_ERROR},
    };
}
