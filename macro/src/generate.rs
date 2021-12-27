use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{ExprPath, Field, Fields, GenericArgument, Ident, ItemStruct, PathArguments, Type};

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
/// All fields must implement `Clone`.
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

/// Internal representation of parsed types
///
/// We either expect fields to have a generic type `T` or `Option<T>`.
enum FieldType {
    Normal(Type),
    Optional(Type),
    Invalid,
}

/// This function takes any [Type] and determines, whether it's an `Option<T>` or just a `T`.
///
/// This detected variant is represented via the [FieldType] enum.
/// Invalid or unsupported types return the `FieldType::Invalid` variant.
fn determine_type(ty: Type) -> FieldType {
    match ty.clone() {
        Type::Path(type_path) => {
            // The path is relative to `Self` and thereby non-optional
            if !type_path.qself.is_none() {
                return FieldType::Normal(ty);
            }

            let path = type_path.path;

            // `Option<T>` shouldn't have a leading colon or multiple segments.
            if path.leading_colon.is_some() || path.segments.len() > 1 {
                return FieldType::Normal(ty);
            }

            // The path should have at least one segment.
            let segment = if let Some(segment) = path.segments.iter().next() {
                segment
            } else {
                return FieldType::Normal(ty);
            };

            // The segment isn't an option.
            if segment.ident != "Option" {
                return FieldType::Normal(ty);
            }

            // Get the angle brackets
            let generic_arg = match &segment.arguments {
                PathArguments::AngleBracketed(params) => {
                    if let Some(arg) = params.args.iter().next() {
                        arg
                    } else {
                        err!(ty, "Option doesn't have a type parameter..");
                        return FieldType::Invalid;
                    }
                }
                _ => {
                    err!(
                        ty,
                        "Unknown path arguments behind Option. Please report this."
                    );
                    return FieldType::Invalid;
                }
            };

            // This argument must be a type:
            match generic_arg {
                GenericArgument::Type(inner_type) => FieldType::Optional(inner_type.clone()),
                _ => {
                    err!(ty, "Option path argument isn't a type.");
                    FieldType::Invalid
                }
            }
        }
        _ => {
            err!(
                ty,
                "Found a non-path type. This isn't supported in struct-merge yet."
            );
            return FieldType::Invalid;
        }
    }
}
