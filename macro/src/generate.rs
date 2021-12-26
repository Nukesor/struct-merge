use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{ExprPath, Field, Fields, Ident, ItemStruct};

/// Return a Tokenstream that contains all implementations to merge `src` into the `dest`
/// struct.
///
/// Known Limitations:
/// - No proper error message on fields with same Tokens, but different generics.
/// - Error, when using different generic aliases that have same type.
/// - Visibility of the `dest` struct isn't taken into account.
///     This will get better, when module resolution is done properly.
///
/// TODO:
/// - [ ] Ensure correct types.
/// - [ ] Support Option<T> merges.
/// - [ ] Check for Options in merge.
/// - [ ] Merge function for owned struct (consumes).
/// - [ ] Merge function that clones fields.
/// - [ ] soft_merge function that only override if Options are `Some`.
pub fn generate_implementations(
    src_ident: Ident,
    dest_path: ExprPath,
    dest: ItemStruct,
    src: ItemStruct,
) -> Option<TokenStream> {
    let dest_fields = match dest.fields {
        Fields::Named(fields) => fields,
        _ => {
            err!(
                dest,
                "struct_merge only works on structs with named fields."
            );
            return None;
        }
    };
    let src_fields = match src.fields {
        Fields::Named(fields) => fields,
        _ => {
            err!(src, "struct_merge only works on structs with named fields.");
            return None;
        }
    };

    let mut similar_fields = Vec::new();
    for src_field in src_fields.named {
        for dest_field in dest_fields.named.clone() {
            if src_field.ident == dest_field.ident {
                similar_fields.push((src_field.clone(), dest_field));
            }
        }
    }

    // In the following, we'll generate all required functions for the `MergeInto` impl.
    // If any of the functions fails to be generated, we skip the impl for this struct.
    // The errors will be generated in the individual token generator functions.
    let mut functions_tokens = proc_macro2::TokenStream::new();

    // Generate the `src.merge_into(&mut target)` function
    match generate_merge_into(dest_path.clone(), similar_fields.clone()) {
        Some(stream) => functions_tokens.extend(vec![stream]),
        None => return None,
    }

    // Generate the `src.merge_into_owned(&mut target)` function
    match generate_merge_into_owned(dest_path.clone(), similar_fields.clone()) {
        Some(stream) => functions_tokens.extend(vec![stream]),
        None => return None,
    }

    Some(
        quote! {
            impl struct_merge::StructMergeInto<#dest_path> for #src_ident {
                #functions_tokens
            }
        }
        .into(),
    )
}

/// Generate the `MergeInto::merge_into` function for the fields of the given structs.
///
/// All fields must implement `Clone`
pub fn generate_merge_into(
    dest_path: ExprPath,
    fields: Vec<(Field, Field)>,
) -> Option<proc_macro2::TokenStream> {
    let mut merge_code = proc_macro2::TokenStream::new();
    for (src_field, dest_field) in fields {
        let src_ident = src_field.ident;
        let dest_ident = dest_field.ident;
        let tokens = quote! {
            dest.#dest_ident = self.#src_ident.clone();
        };
        merge_code.extend(vec![tokens]);
    }

    let merge_code = merge_code.to_token_stream();

    Some(quote! {
        fn merge_into(&self, dest: &mut #dest_path) {
            #merge_code
        }
    })
}

pub fn generate_merge_into_owned(
    dest_path: ExprPath,
    fields: Vec<(Field, Field)>,
) -> Option<proc_macro2::TokenStream> {
    let mut merge_code = proc_macro2::TokenStream::new();
    for (src_field, dest_field) in fields {
        let src_ident = src_field.ident;
        let dest_ident = dest_field.ident;
        let tokens = quote! {
            dest.#dest_ident = self.#src_ident;
        };
        merge_code.extend(vec![tokens]);
    }

    let merge_code = merge_code.to_token_stream();

    Some(quote! {
        fn merge_into_owned(self, dest: &mut #dest_path) {
            #merge_code
        }
    })
}
