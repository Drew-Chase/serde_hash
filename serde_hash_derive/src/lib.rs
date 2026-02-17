#![doc = include_str!("../README.MD")]
use proc_macro::TokenStream;
use quote::quote;
use syn::{GenericArgument, PathArguments, Type};

// --- Shared type detection helpers ---

fn is_numeric_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
            let ident = &type_path.path.segments.first().unwrap().ident;
            matches!(
                ident.to_string().as_str(),
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
            )
        } else {
            false
        }
    } else {
        false
    }
}

fn is_vector_of_numeric(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
            let segment = type_path.path.segments.first().unwrap();
            if segment.ident == "Vec" {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if args.args.len() == 1 {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return is_numeric_type(inner_ty);
                        }
                    }
                }
            }
        }
    }
    false
}

fn is_option_of_numeric(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
            let segment = type_path.path.segments.first().unwrap();
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if args.args.len() == 1 {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return is_numeric_type(inner_ty);
                        }
                    }
                }
            }
        }
    }
    false
}

fn is_option_of_vector_of_numeric(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
            let segment = type_path.path.segments.first().unwrap();
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if args.args.len() == 1 {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return is_vector_of_numeric(inner_ty);
                        }
                    }
                }
            }
        }
    }
    false
}

fn determine_with_path(ty: &Type) -> Option<&'static str> {
    if is_numeric_type(ty) {
        Some("serde_hash::serde_impl::numeric")
    } else if is_vector_of_numeric(ty) {
        Some("serde_hash::serde_impl::vec_numeric")
    } else if is_option_of_numeric(ty) {
        Some("serde_hash::serde_impl::option_numeric")
    } else if is_option_of_vector_of_numeric(ty) {
        Some("serde_hash::serde_impl::option_vec_numeric")
    } else {
        None
    }
}

// --- New #[serde_hash] attribute macro ---

/// Attribute macro that enables `#[serde(hash)]` on struct fields.
///
/// Place this on a struct **above** `#[derive(Serialize, Deserialize)]`.
/// Fields marked with `#[serde(hash)]` (or standalone `#[hash]`) will have their
/// numeric values automatically hash-encoded during serialization and decoded
/// during deserialization. All other serde attributes (`rename`, `alias`, `default`,
/// `skip`, etc.) work normally alongside `hash`.
///
/// # Supported field types
/// - `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
/// - `Vec<T>` where `T` is one of the above
/// - `Option<T>` where `T` is one of the above
/// - `Option<Vec<T>>` where `T` is one of the above
///
/// # Example
/// ```ignore
/// use serde::{Serialize, Deserialize};
/// use serde_hash::serde_hash;
///
/// #[serde_hash]
/// #[derive(Serialize, Deserialize)]
/// pub struct User {
///     #[serde(hash, alias = "identifier")]
///     pub id: u64,
///     #[serde(rename = "user_name")]
///     pub name: String,
///     #[serde(hash)]
///     pub age: Option<u8>,
/// }
/// ```
#[proc_macro_attribute]
pub fn serde_hash(_attr: TokenStream, item: TokenStream) -> TokenStream {
    use syn::{parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Fields, Meta, Token};

    let mut input = parse_macro_input!(item as DeriveInput);

    let fields = match &mut input.data {
        Data::Struct(data) => match &mut data.fields {
            Fields::Named(fields) => &mut fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input.ident,
                    "#[serde_hash] only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "#[serde_hash] can only be applied to structs",
            )
            .to_compile_error()
            .into();
        }
    };

    for field in fields.iter_mut() {
        let mut needs_hash = false;
        let field_ty = field.ty.clone();

        let mut new_attrs = Vec::new();
        for attr in &field.attrs {
            // Standalone #[hash] attribute
            if attr.path().is_ident("hash") {
                needs_hash = true;
                continue;
            }

            // #[serde(...)] attribute — check for `hash` inside
            if attr.path().is_ident("serde") {
                let nested = attr
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);

                if let Ok(metas) = nested {
                    let has_hash = metas
                        .iter()
                        .any(|m| matches!(m, Meta::Path(p) if p.is_ident("hash")));

                    if has_hash {
                        needs_hash = true;
                        // Keep all non-hash meta items
                        let remaining: Vec<_> = metas
                            .iter()
                            .filter(|m| !matches!(m, Meta::Path(p) if p.is_ident("hash")))
                            .collect();

                        if !remaining.is_empty() {
                            // Rebuild #[serde(...)] without hash
                            new_attrs.push(syn::parse_quote!(#[serde(#(#remaining),*)]));
                        }
                        // If hash was the only item, drop the entire attribute
                        continue;
                    }
                }
                // No hash found, keep attribute as-is
                new_attrs.push(attr.clone());
            } else {
                new_attrs.push(attr.clone());
            }
        }

        if needs_hash {
            match determine_with_path(&field_ty) {
                Some(path) => {
                    new_attrs.push(syn::parse_quote!(#[serde(with = #path)]));
                }
                None => {
                    let field_name = field.ident.as_ref().unwrap();
                    return syn::Error::new_spanned(
                        &field.ty,
                        format!(
                            "The `hash` attribute on field '{}' requires a numeric type \
                             (u8, u16, u32, u64, u128, usize), Vec<numeric>, \
                             Option<numeric>, or Option<Vec<numeric>>",
                            field_name
                        ),
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }

        field.attrs = new_attrs;
    }

    quote!(#input).into()
}

// --- Legacy #[hash] passthrough attribute (kept for backward compatibility) ---

#[proc_macro_attribute]
pub fn hash(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// --- Legacy #[derive(HashIds)] (kept for backward compatibility) ---

#[proc_macro_derive(HashIds, attributes(hash))]
pub fn hash_id_derive(input: TokenStream) -> TokenStream {
    use syn::{parse_macro_input, Data, DeriveInput, Fields};

    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let mut errors = Vec::new();

    // Validate #[hash] fields.
    if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            for field in fields.named.iter() {
                let has_hash = field.attrs.iter().any(|attr| attr.path().is_ident("hash"));
                if has_hash {
                    if let Some(field_name) = &field.ident {
                        if !is_numeric_type(&field.ty)
                            && !is_vector_of_numeric(&field.ty)
                            && !is_option_of_numeric(&field.ty)
                            && !is_option_of_vector_of_numeric(&field.ty)
                        {
                            errors.push(quote! {
                                compile_error!(concat!("The #[hash] attribute can only be applied to numeric fields, vectors of numeric fields, or Option types of these, but field '",
                                    stringify!(#field_name),
                                    "' has type '",
                                    stringify!(#field.ty), "'"));
                            });
                        }
                    }
                }
            }
        }
    }

    // Separate fields into different categories based on their types.
    let numeric_hash_fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .filter_map(|field| {
                    let has_hash = field.attrs.iter().any(|attr| attr.path().is_ident("hash"));
                    if has_hash && is_numeric_type(&field.ty) {
                        field.ident.as_ref()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let vector_hash_fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .filter_map(|field| {
                    let has_hash = field.attrs.iter().any(|attr| attr.path().is_ident("hash"));
                    if has_hash && is_vector_of_numeric(&field.ty) {
                        field.ident.as_ref()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let option_numeric_hash_fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .filter_map(|field| {
                    let has_hash = field.attrs.iter().any(|attr| attr.path().is_ident("hash"));
                    if has_hash && is_option_of_numeric(&field.ty) {
                        field.ident.as_ref()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let option_vector_hash_fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .filter_map(|field| {
                    let has_hash = field.attrs.iter().any(|attr| attr.path().is_ident("hash"));
                    if has_hash && is_option_of_vector_of_numeric(&field.ty) {
                        field.ident.as_ref()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let non_hash_fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .filter_map(|field| {
                    let has_hash = field.attrs.iter().any(|attr| attr.path().is_ident("hash"));
                    if !has_hash {
                        field.ident.as_ref()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Get the total number of fields.
    let field_count = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields.named.len()
        } else {
            0
        }
    } else {
        0
    };

    if !errors.is_empty() {
        return TokenStream::from(quote! {
            #(#errors)*
        });
    }

    // Generate code for Serialize and Deserialize.
    let output = quote! {
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
                use serde::ser::SerializeStruct;
                use serde_hash::hashids::encode_single;

                let mut s = serializer.serialize_struct(stringify!(#name), #field_count)?;

                #(
                    s.serialize_field(
                        stringify!(#numeric_hash_fields),
                        &encode_single(self.#numeric_hash_fields as u64)
                    )?;
                )*

                #(
                    {
                        let mut tmp_vec = Vec::new();
                        for v in &self.#vector_hash_fields {
                            tmp_vec.push(encode_single(*v as u64));
                        }
                        s.serialize_field(
                            stringify!(#vector_hash_fields),
                            &tmp_vec
                        )?;
                    }
                )*

                #(
                    {
                        if let Some(value) = self.#option_numeric_hash_fields {
                            s.serialize_field(
                                stringify!(#option_numeric_hash_fields),
                                &Some(encode_single(value as u64))
                            )?;
                        } else {
                            s.serialize_field(
                                stringify!(#option_numeric_hash_fields),
                                &Option::<String>::None
                            )?;
                        }
                    }
                )*

                #(
                    {
                        if let Some(vec_value) = &self.#option_vector_hash_fields {
                            let mut tmp_vec = Vec::new();
                            for v in vec_value {
                                tmp_vec.push(encode_single(*v as u64));
                            }
                            s.serialize_field(
                                stringify!(#option_vector_hash_fields),
                                &Some(tmp_vec)
                            )?;
                        } else {
                            s.serialize_field(
                                stringify!(#option_vector_hash_fields),
                                &Option::<Vec<String>>::None
                            )?;
                        }
                    }
                )*

                #(
                    s.serialize_field(stringify!(#non_hash_fields), &self.#non_hash_fields)?;
                )*

                s.end()
            }
        }

        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                use serde::de::{self, MapAccess, Visitor};
                use std::fmt;
                use serde_hash::hashids::decode_single;

                struct StructVisitor;

                impl<'de> Visitor<'de> for StructVisitor {
                    type Value = #name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(concat!("struct ", stringify!(#name)))
                    }

                    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                    where V: MapAccess<'de> {
                        #(
                            let mut #numeric_hash_fields = None;
                        )*
                        #(
                            let mut #vector_hash_fields = None;
                        )*
                        #(
                            let mut #option_numeric_hash_fields = None;
                        )*
                        #(
                            let mut #option_vector_hash_fields = None;
                        )*
                        #(
                            let mut #non_hash_fields = None;
                        )*

                        while let Some(key) = map.next_key::<String>()? {
                            match key.as_str() {
                                #(
                                    stringify!(#numeric_hash_fields) => {
                                        let hash_str = map.next_value::<String>()?;
                                        let decoded = decode_single(hash_str)
                                            .map_err(|e| de::Error::custom(format!("Failed to decode hash: {}", e)))?;
                                        #numeric_hash_fields = Some(decoded as _);
                                    },
                                )*
                                #(
                                    stringify!(#vector_hash_fields) => {
                                        let hash_vec = map.next_value::<Vec<String>>()?;
                                        let mut decoded_vec = Vec::new();
                                        for hash in hash_vec {
                                            let decoded = decode_single(hash)
                                                .map_err(|e| de::Error::custom(format!("Failed to decode hash: {}", e)))?;
                                            decoded_vec.push(decoded as _);
                                        }
                                        #vector_hash_fields = Some(decoded_vec);
                                    },
                                )*
                                #(
                                    stringify!(#option_numeric_hash_fields) => {
                                        let option_hash = map.next_value::<Option<String>>()?;
                                        if let Some(hash_str) = option_hash {
                                            let decoded = decode_single(hash_str)
                                                .map_err(|e| de::Error::custom(format!("Failed to decode hash: {}", e)))?;
                                            #option_numeric_hash_fields = Some(Some(decoded as _));
                                        } else {
                                            #option_numeric_hash_fields = Some(None);
                                        }
                                    },
                                )*
                                #(
                                    stringify!(#option_vector_hash_fields) => {
                                        let option_hash_vec = map.next_value::<Option<Vec<String>>>()?;
                                        if let Some(hash_vec) = option_hash_vec {
                                            let mut decoded_vec = Vec::new();
                                            for hash in hash_vec {
                                                let decoded = decode_single(hash)
                                                    .map_err(|e| de::Error::custom(format!("Failed to decode hash: {}", e)))?;
                                                decoded_vec.push(decoded as _);
                                            }
                                            #option_vector_hash_fields = Some(Some(decoded_vec));
                                        } else {
                                            #option_vector_hash_fields = Some(None);
                                        }
                                    },
                                )*
                                #(
                                    stringify!(#non_hash_fields) => {
                                        #non_hash_fields = Some(map.next_value()?);
                                    },
                                )*
                                _ => {
                                    let _ = map.next_value::<de::IgnoredAny>()?;
                                }
                            }
                        }

                        #(
                            let #numeric_hash_fields = #numeric_hash_fields.ok_or_else(||
                                de::Error::missing_field(stringify!(#numeric_hash_fields))
                            )?;
                        )*
                        #(
                            let #vector_hash_fields = #vector_hash_fields.ok_or_else(||
                                de::Error::missing_field(stringify!(#vector_hash_fields))
                            )?;
                        )*
                        #(
                            let #option_numeric_hash_fields = #option_numeric_hash_fields.ok_or_else(||
                                de::Error::missing_field(stringify!(#option_numeric_hash_fields))
                            )?;
                        )*
                        #(
                            let #option_vector_hash_fields = #option_vector_hash_fields.ok_or_else(||
                                de::Error::missing_field(stringify!(#option_vector_hash_fields))
                            )?;
                        )*
                        #(
                            let #non_hash_fields = #non_hash_fields.ok_or_else(||
                                de::Error::missing_field(stringify!(#non_hash_fields))
                            )?;
                        )*

                        Ok(#name {
                            #(
                                #numeric_hash_fields,
                            )*
                            #(
                                #vector_hash_fields,
                            )*
                            #(
                                #option_numeric_hash_fields,
                            )*
                            #(
                                #option_vector_hash_fields,
                            )*
                            #(
                                #non_hash_fields,
                            )*
                        })
                    }
                }

                deserializer.deserialize_map(StructVisitor)
            }
        }
    };

    output.into()
}
