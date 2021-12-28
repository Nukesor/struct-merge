use proc_macro::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{ExprPath, Fields, GenericArgument, Ident, ItemStruct, PathArguments, Type};

use crate::Mode;

macro_rules! equal_type_or_continue {
    ($src_type:ident, $dest_type:ident, $type:expr) => {
        if !is_equal_type(&$src_type, &$dest_type) {
            err!(
                $src_type,
                "{}Type '{} cannot be merged into field of type '{}'.",
                $type,
                $src_type.to_token_stream(),
                $dest_type.to_token_stream()
            );
            continue;
        }
    };
}

mod borrowed;
mod owned;

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
pub(crate) fn generate_impl(
    mode: &Mode,
    src_ident: Ident,
    dest_path: ExprPath,
    src: ItemStruct,
    dest: ItemStruct,
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
    match mode {
        &Mode::Owned => owned::generate_impl_owned(src_ident, dest_path, similar_fields),
        &Mode::Borrowed => borrowed::generate_impl_borrowed(src_ident, dest_path, similar_fields),
    }
}

/// Check whether two given [Type]s are of the same type.
/// If they aren't, an error is added to the src_type and the function returns `false`.
fn is_equal_type(src_type: &Type, dest_type: &Type) -> bool {
    // This is a reather stupid way of comparing equality, but it has to do for now.
    if src_type.to_token_stream().to_string() != dest_type.to_token_stream().to_string() {
        return false;
    }

    true
}

/// Internal representation of parsed types
///
/// We either expect fields to have a generic type `T` or `Option<T>`.
enum FieldType {
    Normal(Type),
    Optional { inner: Type, outer: Type },
    Invalid,
}

/// This function takes any [Type] and determines, whether it's an `Option<T>` or just a `T`.
///
/// This detected variant is represented via the [FieldType] enum.
/// Invalid or unsupported types return the `FieldType::Invalid` variant.
fn determine_field_type(ty: Type) -> FieldType {
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
                GenericArgument::Type(inner_type) => FieldType::Optional {
                    inner: inner_type.clone(),
                    outer: ty,
                },
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
