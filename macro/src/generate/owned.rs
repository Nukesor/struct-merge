use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{ExprPath, Field, Ident};

use super::*;

/// Generate the implementation of [struct_merge::StructMerge] for given structs.
pub fn impl_owned(
    src_ident: Ident,
    dest_path: ExprPath,
    fields: Vec<(Field, Field)>,
) -> TokenStream {
    let mut functions_tokens = TokenStream::new();

    let stream = merge(src_ident.clone(), fields.clone());
    functions_tokens.extend(vec![stream]);

    let stream = merge_soft(src_ident.clone(), fields);
    functions_tokens.extend(vec![stream]);

    quote! {
        impl struct_merge::StructMerge<#src_ident> for #dest_path {
            #functions_tokens
        }
    }
}

/// Generate the [struct_merge::StructMerge::merge] function for the given structs.
fn merge(src_ident: Ident, fields: Vec<(Field, Field)>) -> TokenStream {
    let mut merge_code = TokenStream::new();
    for (src_field, dest_field) in fields {
        let src_ident = src_field.ident;
        let dest_ident = dest_field.ident;

        // Find out, whether the fields are optional or not.
        let src_field_type = determine_field_type(src_field.ty);
        let dest_field_type = determine_field_type(dest_field.ty);

        let snippet = match (src_field_type, dest_field_type) {
            // Both fields have the same type
            (FieldType::Normal(src_type), FieldType::Normal(dest_type)) => {
                equal_type_or_continue!(
                    src_type,
                    dest_type,
                    "",
                    quote! {
                        self.#dest_ident = src.#src_ident;
                    }
                )
            }
            // The src is optional and needs to be `Some(T)` to be merged.
            (
                FieldType::Optional {
                    inner: src_type, ..
                },
                FieldType::Normal(dest_type),
            ) => {
                equal_type_or_continue!(
                    src_type,
                    dest_type,
                    "Inner",
                    quote! {
                        if let Some(value) = src.#src_ident {
                            self.#dest_ident = value;
                        }
                    }
                )
            }
            // The dest is optional and needs to be wrapped in `Some(T)` to be merged.
            (
                FieldType::Normal(src_type),
                FieldType::Optional {
                    inner: dest_type, ..
                },
            ) => {
                equal_type_or_continue!(
                    src_type,
                    dest_type,
                    "",
                    quote! {
                        self.#dest_ident = Some(src.#src_ident);
                    }
                )
            }
            // Both fields are optional. It can now be either of these:
            // - (Option<T>, Option<T>)
            // - (Option<Option<T>>, Option<T>)
            // - (Option<T>, Option<Option<T>>)
            (
                FieldType::Optional {
                    inner: inner_src_type,
                    outer: outer_src_type,
                },
                FieldType::Optional {
                    inner: inner_dest_type,
                    outer: outer_dest_type,
                },
            ) => {
                // Handling the (Option<T>, Option<T>) case
                if is_equal_type(&inner_src_type, &inner_dest_type) {
                    quote! {
                        self.#dest_ident = src.#src_ident;
                    }
                // Handling the (Option<Option<<T>>, Option<T>) case
                } else if is_equal_type(&inner_src_type, &outer_dest_type) {
                    quote! {
                        if let Some(value) = src.#src_ident {
                            self.#dest_ident = value;
                        }
                    }
                // Handling the (Option<<T>, Option<Option<T>)> case
                } else {
                    equal_type_or_continue!(
                        outer_src_type,
                        inner_dest_type,
                        "",
                        quote! {
                            self.#dest_ident = Some(src.#src_ident);
                        }
                    )
                }
            }
            // Skip anything where either of the fields are invalid
            (FieldType::Invalid, _) | (_, FieldType::Invalid) => continue,
        };

        merge_code.extend(vec![snippet]);
    }

    let merge_code = merge_code.to_token_stream();

    quote! {
        fn merge(&mut self, src: #src_ident) {
            #merge_code
        }
    }
}

/// Generate the [struct_merge::StructMerge::merge_soft] function for the given structs.
fn merge_soft(src_ident: Ident, fields: Vec<(Field, Field)>) -> TokenStream {
    let mut merge_code = TokenStream::new();
    for (src_field, dest_field) in fields {
        let src_ident = src_field.ident;
        let dest_ident = dest_field.ident;

        // Find out, whether the fields are optional or not.
        let src_field_type = determine_field_type(src_field.ty);
        let dest_field_type = determine_field_type(dest_field.ty);

        let snippet = match (src_field_type, dest_field_type) {
            // Soft merge only applies if the dest field is `Optional`.
            (FieldType::Normal(_), FieldType::Normal(_))
            | (FieldType::Optional { .. }, FieldType::Normal(_)) => continue,
            // The dest is optional and needs to be wrapped in `Some(T)` to be merged.
            (
                FieldType::Normal(src_type),
                FieldType::Optional {
                    inner: dest_type, ..
                },
            ) => {
                equal_type_or_continue!(
                    src_type,
                    dest_type,
                    "",
                    quote! {
                        if self.#dest_ident.is_none() {
                            self.#dest_ident = Some(src.#src_ident);
                        }
                    }
                )
            }
            // Both fields are optional. It can now be either of these:
            // - (Option<T>, Option<T>)
            // - (Option<Option<T>>, Option<T>)
            // - (Option<T>, Option<Option<T>>)
            (
                FieldType::Optional {
                    inner: inner_src_type,
                    outer: outer_src_type,
                },
                FieldType::Optional {
                    inner: inner_dest_type,
                    outer: outer_dest_type,
                },
            ) => {
                // Handling the (Option<T>, Option<T>) case
                if is_equal_type(&inner_src_type, &inner_dest_type) {
                    quote! {
                        if self.#dest_ident.is_none() {
                            self.#dest_ident = src.#src_ident;
                        }
                    }
                // Handling the (Option<Option<<T>>, Option<T>) case
                } else if is_equal_type(&inner_src_type, &outer_dest_type) {
                    quote! {
                        if let Some(value) = src.#src_ident {
                            if self.#dest_ident.is_none() {
                                self.#dest_ident = value;
                            }
                        }
                    }
                // Handling the (Option<<T>, Option<Option<T>)> case
                } else {
                    equal_type_or_continue!(
                        outer_src_type,
                        inner_dest_type,
                        "",
                        quote! {
                            if self.#dest_ident.is_none() {
                                self.#dest_ident = Some(src.#src_ident);
                            }
                        }
                    )
                }
            }
            // Skip anything where either of the fields are invalid
            (FieldType::Invalid, _) | (_, FieldType::Invalid) => continue,
        };

        merge_code.extend(vec![snippet]);
    }

    let merge_code = merge_code.to_token_stream();

    quote! {
        fn merge_soft(&mut self, src: #src_ident) {
            #merge_code
        }
    }
}
