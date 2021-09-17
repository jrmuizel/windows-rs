use super::*;

pub fn gen_pwstr() -> TokenStream {
    quote! {
        #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
        #[repr(transparent)]
        pub struct PWSTR(pub *mut u16);
        impl ::std::default::Default for PWSTR {
            fn default() -> Self {
                Self(::std::ptr::null_mut())
            }
        }
        unsafe impl ::windows::Abi for PWSTR {
            type Abi = Self;
            type DefaultType = Self;

            unsafe fn drop_param(param: &mut ::windows::Param<'_, Self>) {
                if let ::windows::Param::Boxed(value) = param {
                    if !value.0.is_null() {
                        unsafe { ::std::boxed::Box::from_raw(value.0); }
                    }
                }
            }
        }
        impl<'a> ::windows::IntoParam<'a, PWSTR> for &str {
            fn into_param(self) -> ::windows::Param<'a, PWSTR> {
                ::windows::Param::Boxed(PWSTR(::std::boxed::Box::<[u16]>::into_raw(self.encode_utf16().chain(::std::iter::once(0)).collect::<std::vec::Vec<u16>>().into_boxed_slice()) as _))
            }
        }
        impl<'a> ::windows::IntoParam<'a, PWSTR> for String {
            fn into_param(self) -> ::windows::Param<'a, PWSTR> {
                // TODO: call variant above
                ::windows::Param::Boxed(PWSTR(::std::boxed::Box::<[u16]>::into_raw(self.encode_utf16().chain(::std::iter::once(0)).collect::<std::vec::Vec<u16>>().into_boxed_slice()) as _))
            }
        }
        #[cfg(windows)]
        impl<'a> ::windows::IntoParam<'a, PWSTR> for &::std::ffi::OsStr {
            fn into_param(self) -> ::windows::Param<'a, PWSTR> {
                use std::os::windows::ffi::OsStrExt;
                ::windows::Param::Boxed(PWSTR(::std::boxed::Box::<[u16]>::into_raw(self.encode_wide().chain(::std::iter::once(0)).collect::<std::vec::Vec<u16>>().into_boxed_slice()) as _))
            }
        }
        #[cfg(windows)]
        impl<'a> ::windows::IntoParam<'a, PWSTR> for ::std::ffi::OsString {
            fn into_param(self) -> ::windows::Param<'a, PWSTR> {
                use std::os::windows::ffi::OsStrExt;
                ::windows::Param::Boxed(PWSTR(::std::boxed::Box::<[u16]>::into_raw(self.encode_wide().chain(::std::iter::once(0)).collect::<std::vec::Vec<u16>>().into_boxed_slice()) as _))
            }
        }
    }
}
