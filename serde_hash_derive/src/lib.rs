use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_attribute]
pub fn hash(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
#[proc_macro_derive(HashIds, attributes(hash))]
pub fn hash_id_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Track any type errors for diagnostics
    let mut errors = Vec::new();

    // Extract field names that have the #[hash] attribute and validate their types
    let hash_fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .filter_map(|field| {
                    let has_hash = field.attrs.iter().any(|attr| attr.path().is_ident("hash"));
                    if has_hash {
                        // Check if the field has a numeric type
                        if let Some(field_name) = &field.ident {
                            if let syn::Type::Path(type_path) = &field.ty {
                                let type_str = quote!(#type_path).to_string();
                                // Check for numeric types
                                if !["u8", "u16", "u32", "u64", "u128", "usize"]
                                    .iter()
                                    .any(|&t| type_str.contains(t)) {
                                    errors.push(quote! {
                                        compile_error!(concat!("The #[hash] attribute can only be applied to numeric fields, but field '",
                                            stringify!(#field_name),
                                            "' has type '",
                                            stringify!(#type_path), "'"));
                                    });
                                    return None;
                                }
                            }
                        }
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

    // Extract field names that don't have the #[hash] attribute
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

    // Get the total number of fields
    let field_count = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields.named.len()
        } else {
            0
        }
    } else {
        0
    };

    // If there are type errors, return them instead of the implementation
    if !errors.is_empty() {
        return TokenStream::from(quote! {
            #(#errors)*
        });
    }

    // Generate code for serialization and deserialization
    let output = quote! {
        // Implement Serialize manually to handle hash fields
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeStruct;
                use serde_hash::hashids::encode_single;

                // Create serializer with correct field count
                let mut s = serializer.serialize_struct(stringify!(#name), #field_count)?;

                // For hash fields, encode the value using encode_single after converting to u64
                #(
                    s.serialize_field(
                        stringify!(#hash_fields),
                        &encode_single(self.#hash_fields as u64)
                    )?;
                )*

                // For non-hash fields, serialize normally
                #(
                    s.serialize_field(stringify!(#non_hash_fields), &self.#non_hash_fields)?;
                )*

                s.end()
            }
        }

        // Implement Deserialize manually to handle hash fields
        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
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
                    where
                        V: MapAccess<'de>,
                    {
                        #(
                            let mut #hash_fields = None;
                        )*

                        #(
                            let mut #non_hash_fields = None;
                        )*

                        while let Some(key) = map.next_key::<String>()? {
                            match key.as_str() {
                                #(
                                    stringify!(#hash_fields) => {
                                        let hash_str = map.next_value::<String>()?;
                                        let decoded = decode_single(hash_str)
                                            .map_err(|e| de::Error::custom(format!("Failed to decode hash: {}", e)))?;
                                        #hash_fields = Some(decoded as _);
                                    },
                                )*

                                #(
                                    stringify!(#non_hash_fields) => {
                                        #non_hash_fields = Some(map.next_value()?);
                                    },
                                )*

                                _ => {
                                    // Skip unknown fields
                                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                                }
                            }
                        }

                        #(
                            let #hash_fields = #hash_fields.ok_or_else(||
                                de::Error::missing_field(stringify!(#hash_fields))
                            )?;
                        )*

                        #(
                            let #non_hash_fields = #non_hash_fields.ok_or_else(||
                                de::Error::missing_field(stringify!(#non_hash_fields))
                            )?;
                        )*

                        Ok(#name {
                            #(
                                #hash_fields,
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
