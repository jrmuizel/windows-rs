use super::*;

// TODO: need to split win32 and winrt structs as their signatures are different and win32 structs also include unions and they are
// radically different.

pub fn gen_struct(def: &TypeDef, gen: &Gen) -> TokenStream {
    gen_struct_with_name(def, def.name(), gen)
}

fn gen_struct_with_name(def: &TypeDef, struct_name: &str, gen: &Gen) -> TokenStream {
    if let Some(replacement) = gen_replacement(def) {
        return replacement;
    }

    let name = to_ident(struct_name);

    if def.is_handle() {
        let signature = def.fields().next().map(|field| field.signature()).unwrap();
        let signature = gen_sig(&signature, gen);

        let convertible = if let Some(dependency) = def.is_convertible_to() {
            let dependency = gen_type_name(&dependency, gen);

            quote! {
                impl<'a> ::windows::IntoParam<'a, #dependency> for #name {
                    fn into_param(self) -> ::windows::Param<'a, #dependency> {
                        ::windows::Param::Owned(#dependency(self.0))
                    }
                }
            }
        } else {
            quote! {}
        };

        return quote! {
            #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
            #[repr(transparent)]
            pub struct #name(pub #signature);
            impl ::std::default::Default for #name {
                fn default() -> Self {
                    unsafe { ::std::mem::zeroed() }
                }
            }
            unsafe impl ::windows::Handle for #name {}
            unsafe impl ::windows::Abi for #name {
                type Abi = Self;
                type DefaultType = Self;
            }
            #convertible
        };
    }

    if let Some(guid) = Guid::from_attributes(def.attributes()) {
        let guid = gen_guid(&guid);

        return quote! {
            pub const #name: ::windows::Guid = ::windows::Guid::from_values(#guid);
        };
    }

    let fields: Vec<(Field, Signature, TokenStream)> = def
        .fields()
        .filter_map(move |f| {
            if f.is_literal() {
                None
            } else {
                let signature = f.signature();
                let name = f.name();
                Some((f, signature, to_ident(name)))
            }
        })
        .collect();

    if fields.is_empty() {
        return quote! {
            #[repr(C)]
            #[derive(::std::clone::Clone, ::std::default::Default, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::marker::Copy)]
            pub struct #name(pub u8);
        };
    }

    let is_winrt = def.is_winrt();
    let is_union = def.is_explicit();
    let layout = def.class_layout();
    let is_packed = def.is_packed();

    let repr = if let Some(layout) = &layout {
        let packing = Literal::u32_unsuffixed(layout.packing_size());
        quote! { #[repr(C, packed(#packing))] }
    } else {
        quote! { #[repr(C)] }
    };

    // TODO: add test for Windows.Win32.Security.TRUSTEE_A
    let has_union = fields
        .iter()
        .any(|(_, signature, _)| signature.has_explicit());

    let has_complex_array = fields
        .iter()
        .any(|(_, signature, _)| match &signature.kind {
            ElementType::Array((signature, _)) => {
                !signature.is_blittable() || signature.kind.is_nullable()
            }
            _ => false,
        });

    let runtime_type = if is_winrt {
        let signature = Literal::byte_string(def.type_signature().as_bytes());

        quote! {
            unsafe impl ::windows::RuntimeType for #name {
                const SIGNATURE: ::windows::ConstBuffer = ::windows::ConstBuffer::from_slice(#signature);
            }
        }
    } else {
        quote! {}
    };

    let clone_or_copy = if def.is_blittable() {
        quote! {
            #[derive(::std::clone::Clone, ::std::marker::Copy)]
        }
    } else if is_union || has_union || is_packed {
        quote! {
            impl ::std::clone::Clone for #name {
                fn clone(&self) -> Self {
                    // TODO: this can transmute for blittable but not non-blittable structs
                    unimplemented!()
                }
            }
        }
    } else {
        quote! {
            #[derive(::std::clone::Clone)]
        }
    };

    let body = {
        let fields = fields.iter().map(|(_, signature, name)| {
            let kind = if is_union {
                gen_abi_sig(signature, gen)
            } else {
                gen_sig(signature, gen)
            };

            quote! {
                pub #name: #kind
            }
        });

        quote! {
            { #(#fields),* }
        }
    };

    let struct_or_union = if is_union {
        quote! { union }
    } else {
        quote! { struct }
    };

    let abi = if def.is_blittable() {
        quote! {
            unsafe impl ::windows::Abi for #name {
                type Abi = Self;
                type DefaultType = Self;
            }
        }
    } else {
        quote! {
            unsafe impl ::windows::Abi for #name {
                type Abi = ::std::mem::ManuallyDrop<Self>;
                type DefaultType = Self;
            }
        }
    };

    let constants = def.fields().filter_map(|f| {
        if f.is_literal() {
            if let Some(constant) = f.constant() {
                let name = to_ident(f.name());
                let value = gen_constant_type_value(&constant.value());

                return Some(quote! {
                    pub const #name: #value;
                });
            }
        }

        None
    });

    let compare = if is_union | has_union | has_complex_array | is_packed {
        quote! {
            impl ::std::cmp::PartialEq for #name {
                fn eq(&self, _other: &Self) -> bool {
                    // TODO: figure out how to compare complex structs
                    unimplemented!()
                }
            }
            impl ::std::cmp::Eq for #name {}
        }
    } else {
        let compare = fields.iter().map(|(_, signature, name)| {
            let is_callback = signature.kind.is_callback();

            if is_callback && signature.pointers == 0 {
                quote! {
                    self.#name.map(|f| f as usize) == other.#name.map(|f| f as usize)
                }
            } else {
                quote! {
                    self.#name == other.#name
                }
            }
        });

        if layout.is_some() {
            quote! {
                impl ::std::cmp::PartialEq for #name {
                    fn eq(&self, other: &Self) -> bool {
                        unsafe { #(#compare)&&* }
                    }
                }
                impl ::std::cmp::Eq for #name {}
            }
        } else {
            quote! {
                impl ::std::cmp::PartialEq for #name {
                    fn eq(&self, other: &Self) -> bool {
                        #(#compare)&&*
                    }
                }
                impl ::std::cmp::Eq for #name {}
            }
        }
    };

    let debug = if is_union || has_union || has_complex_array || is_packed {
        quote! {}
    } else {
        let debug_name = def.name();

        let debug_fields = fields.iter().filter_map(|(_, signature, name)| {
            // TODO: there must be a simpler way to implement Debug just to exclude this type.
            if signature.kind.is_callback() {
                return None;
            }

            if let ElementType::Array((kind, _)) = &signature.kind {
                if kind.kind.is_callback() {
                    return None;
                }
            }

            let field = name.as_str();

            Some(quote! {
                .field(#field, &self.#name)
            })
        });

        if layout.is_some() {
            quote! {
                impl ::std::fmt::Debug for #name {
                    fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        unsafe {
                            fmt.debug_struct(#debug_name)
                                #(#debug_fields)*
                                .finish()
                        }
                    }
                }
            }
        } else {
            quote! {
                impl ::std::fmt::Debug for #name {
                    fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        fmt.debug_struct(#debug_name)
                            #(#debug_fields)*
                            .finish()
                    }
                }
            }
        }
    };

    let extensions = gen_extensions(def);
    let nested_types = gen_nested_types(struct_name, def, gen);

    quote! {
        #clone_or_copy
        #repr
        pub #struct_or_union #name #body
        impl #name {
            #(#constants)*
        }
        impl ::std::default::Default for #name {
            fn default() -> Self {
                unsafe { ::std::mem::zeroed() }
            }
        }
        #debug
        #compare
        #abi
        #runtime_type
        #extensions
        #nested_types
    }
}

fn gen_replacement(def: &TypeDef) -> Option<TokenStream> {
    match def.type_name() {
        TypeName::BOOL => Some(gen_bool32()),
        TypeName::PWSTR => Some(gen_pwstr()),
        TypeName::PSTR => Some(gen_pstr()),
        TypeName::BSTR => Some(gen_bstr()),
        TypeName::NTSTATUS => Some(gen_ntstatus()),
        TypeName::HANDLE => Some(gen_handle()),
        _ => None,
    }
}

fn gen_extensions(def: &TypeDef) -> TokenStream {
    match def.type_name() {
        TypeName::TimeSpan => gen_timespan(),
        TypeName::Vector2 => gen_vector2(),
        TypeName::Vector3 => gen_vector3(),
        TypeName::Vector4 => gen_vector4(),
        TypeName::Matrix3x2 => gen_matrix3x2(),
        TypeName::Matrix4x4 => gen_matrix4x4(),
        _ => TokenStream::new(),
    }
}

fn gen_nested_types<'a>(
    enclosing_name: &'a str,
    enclosing_type: &'a TypeDef,
    gen: &Gen,
) -> TokenStream {
    if let Some(nested_types) = enclosing_type.nested_types() {
        nested_types
            .iter()
            .enumerate()
            .map(|(index, (_, nested_type))| {
                let nested_name = format!("{}_{}", enclosing_name, index);
                gen_struct_with_name(nested_type, &nested_name, gen)
            })
            .collect()
    } else {
        TokenStream::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature() {
        let t = TypeReader::get().expect_type_def(TypeName::new("Windows.Foundation", "Point"));
        assert_eq!(t.type_signature(), "struct(Windows.Foundation.Point;f4;f4)");
    }

    #[test]
    fn test_fields() {
        let t = TypeReader::get().expect_type_def(TypeName::new(
            "Windows.Win32.Graphics.Dxgi",
            "DXGI_FRAME_STATISTICS_MEDIA",
        ));
        let f: Vec<Field> = t.fields().collect();
        assert_eq!(f.len(), 7);

        assert_eq!(f[0].name(), "PresentCount");
        assert_eq!(f[1].name(), "PresentRefreshCount");
        assert_eq!(f[2].name(), "SyncRefreshCount");
        assert_eq!(f[3].name(), "SyncQPCTime");
        assert_eq!(f[4].name(), "SyncGPUTime");
        assert_eq!(f[5].name(), "CompositionMode");
        assert_eq!(f[6].name(), "ApprovedPresentDuration");

        assert!(f[0].signature().kind == ElementType::U32);
        assert!(f[1].signature().kind == ElementType::U32);
        assert!(f[2].signature().kind == ElementType::U32);
        assert!(f[3].signature().kind == ElementType::I64);
        assert!(f[4].signature().kind == ElementType::I64);
        assert!(f[6].signature().kind == ElementType::U32);
    }

    #[test]
    fn test_blittable() {
        assert!(TypeReader::get()
            .expect_type_def(TypeName::new("Windows.Foundation", "Point"))
            .is_blittable(),);
        assert!(!TypeReader::get()
            .expect_type_def(TypeName::new("Windows.UI.Xaml.Interop", "TypeName"))
            .is_blittable(),);
    }
}
