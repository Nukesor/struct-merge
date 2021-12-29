use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Field;

use super::*;

/// Generate the implementation of [struct_merge::StructMergeRef] for given structs.
pub(crate) fn impl_borrowed(params: &Parameters, fields: Vec<(Field, Field)>) -> TokenStream {
    let mut functions_tokens = TokenStream::new();

    let stream = merge_ref(params, fields.clone());
    functions_tokens.extend(vec![stream]);

    let stream = merge_ref_soft(params, fields);
    functions_tokens.extend(vec![stream]);

    let src_ident = &params.src_struct.ident;
    let target_path = &params.target_path;
    quote! {
        impl struct_merge::StructMergeIntoRef<#target_path> for #src_ident {
            #functions_tokens
        }
    }
}

/// Generate the [struct_merge::StructMergeRef::merge_ref] function for given structs.
///
/// All fields must implement `Clone`.
fn merge_ref(params: &Parameters, fields: Vec<(Field, Field)>) -> TokenStream {
    let mut merge_code = TokenStream::new();
    for (src_field, target_field) in fields {
        let src_field_ident = src_field.ident;
        let target_field_ident = target_field.ident;

        // Find out, whether the fields are optional or not.
        let src_field_type = determine_field_type(src_field.ty);
        let target_field_type = determine_field_type(target_field.ty);

        let snippet = match (src_field_type, target_field_type) {
            // Both fields have the same type
            (FieldType::Normal(src_type), FieldType::Normal(target_type)) => {
                equal_type_or_continue!(
                    src_type,
                    target_type,
                    "",
                    quote! {
                        target.#target_field_ident = self.#src_field_ident.clone();
                    }
                )
            }
            // The src is optional and needs to be `Some(T)` to be merged.
            (
                FieldType::Optional {
                    inner: src_type, ..
                },
                FieldType::Normal(target_type),
            ) => {
                equal_type_or_continue!(
                    src_type,
                    target_type,
                    "Inner ",
                    quote! {
                        if let Some(value) = self.#src_field_ident.as_ref() {
                            target.#target_field_ident = value.clone();
                        }
                    }
                )
            }
            // The target is optional and needs to be wrapped in `Some(T)` to be merged.
            (
                FieldType::Normal(src_type),
                FieldType::Optional {
                    inner: target_type, ..
                },
            ) => {
                equal_type_or_continue!(
                    src_type,
                    target_type,
                    "",
                    quote! {
                        self.#target_field_ident = Some(src.#src_field_ident.clone());
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
                    inner: inner_target_type,
                    outer: outer_target_type,
                },
            ) => {
                // Handling the (Option<T>, Option<T>) case
                if is_equal_type(&inner_src_type, &inner_target_type) {
                    quote! {
                        target.#target_field_ident = self.#src_field_ident.clone();
                    }
                // Handling the (Option<Option<<T>>, Option<T>) case
                } else if is_equal_type(&inner_src_type, &outer_target_type) {
                    quote! {
                        if let Some(value) = self.#src_field_ident.as_ref() {
                            target.#target_field_ident = value.clone();
                        }
                    }
                // Handling the (Option<<T>, Option<Option<T>)> case
                } else {
                    equal_type_or_continue!(
                        outer_src_type,
                        inner_target_type,
                        "",
                        quote! {
                            target.#target_field_ident = Some(self.#src_field_ident.clone());
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

    let target_path = &params.target_path;
    quote! {
        fn merge_into_ref(&self, target: &mut #target_path) {
            #merge_code
        }
    }
}

/// Generate the [struct_merge::StructMergeRef::merge_ref_soft] function for given structs.
///
/// All fields must implement `Clone`.
fn merge_ref_soft(params: &Parameters, fields: Vec<(Field, Field)>) -> TokenStream {
    let mut merge_code = TokenStream::new();
    for (src_field, target_field) in fields {
        let src_field_ident = src_field.ident;
        let target_field_ident = target_field.ident;

        // Find out, whether the fields are optional or not.
        let src_field_type = determine_field_type(src_field.ty);
        let target_field_type = determine_field_type(target_field.ty);

        let snippet = match (src_field_type, target_field_type) {
            // Soft merge only applies if the target field is `Optional`.
            (FieldType::Normal(_), FieldType::Normal(_))
            | (FieldType::Optional { .. }, FieldType::Normal(_)) => continue,
            // The target is optional and needs to be wrapped in `Some(T)` to be merged.
            (
                FieldType::Normal(src_type),
                FieldType::Optional {
                    inner: target_type, ..
                },
            ) => {
                equal_type_or_continue!(
                    src_type,
                    target_type,
                    "",
                    quote! {
                        if target.#target_field_ident.is_none() {
                            target.#target_field_ident = Some(self.#src_field_ident.clone());
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
                    inner: inner_target_type,
                    outer: outer_target_type,
                },
            ) => {
                // Handling the (Option<T>, Option<T>) case
                if is_equal_type(&inner_src_type, &inner_target_type) {
                    quote! {
                        if target.#target_field_ident.is_none() {
                            target.#target_field_ident = self.#src_field_ident.clone();
                        }
                    }
                // Handling the (Option<Option<<T>>, Option<T>) case
                } else if is_equal_type(&inner_src_type, &outer_target_type) {
                    quote! {
                        if let Some(value) = self.#src_field_ident.as_ref() {
                            if target.#target_field_ident.is_none() {
                                target.#target_field_ident = value.clone();
                            }
                        }
                    }
                // Handling the (Option<<T>, Option<Option<T>)> case
                } else {
                    equal_type_or_continue!(
                        outer_src_type,
                        inner_target_type,
                        "",
                        quote! {
                            if target.#target_field_ident.is_none() {
                                target.#target_field_ident = Some(self.#src_field_ident.clone());
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

    let target_path = &params.target_path;
    quote! {
        fn merge_into_ref_soft(&self, target: &mut #target_path) {
            #merge_code
        }
    }
}
