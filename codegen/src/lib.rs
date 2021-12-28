use generate::generate_impl;
use module::get_struct_from_path;
use path::{get_root_src_path, parse_input_paths};
use proc_macro::TokenStream;
use syn::{parse_macro_input, Expr, ItemStruct};

/// Helper macro, which attaches an error to a given span.
macro_rules! err {
    ($span:ident, $($text:expr),*) => {
        {
            let message = format!($($text,)*);
            let span = $span.span();
            quote::quote_spanned!( span => compile_error!(#message); )
        }
    }
}

// Uncomment this as soon as proc_macro_diagnostic land in stable.
//
//#![feature(proc_macro_diagnostic)]
///// Helper macro, which attaches an error to a given span.
//macro_rules! err {
//    ($span:ident, $($text:expr),*) => {
//        $span.span()
//            .unwrap()
//            .error(format!($($text,)*))
//            .emit();
//    }
//}

/// Helper macro, which takes a result.
/// Ok(T) => simply return the T
/// Err(err) => Emits an compiler error on the given span with the provided error message.
///             Also returns early with `None`.
///             `None` is used throughout this crate as a gracefull failure.
///             That way all code that can be created is being generated and the user sees all
///             errors without the macro code panicking.
macro_rules! ok_or_err_return {
    ($expr:expr, $span:ident, $($text:expr),*) => {
        match $expr {
            Ok(result) => result,
            Err(error) =>  {
                return Err(err!($span, $($text,)* error));
            }
        }
    }
}

mod generate;
mod module;
mod path;

/// Implement the `struct_merge::StructMerge<T>` trait for all given targets.
///
/// The targets struct paths have to be
/// - absolute
/// - relative to the current crate
/// - contained in this crate
///
/// Eiter a single struct or a list of structs can be provided.
/// `StructMerge<T>` will then be implemented on each given target struct.
///
/// Examples:
/// - `#[struct_merge(crate::structs::Base)]`
/// - `#[struct_merge([crate::structs::Base, crate:structs::Other])]`
///
/// `struct.rs`
/// ```ignore
/// use struct_merge::struct_merge;
///
/// pub struct Base {
///     pub test: String,
/// }
///
/// pub struct Other {
///     pub test: String,
/// }
///
/// #[struct_merge([crate::structs::Base, crate:structs::Other])]
/// pub struct Test {
///     pub test: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn struct_merge(args: TokenStream, struct_ast: TokenStream) -> TokenStream {
    struct_merge_base(args, struct_ast, Mode::Owned)
}

/// Implement the `struct_merge::StructMergeRef<T>` trait for all given targets.
///
/// The targets struct paths have to be
/// - absolute
/// - relative to the current crate
/// - contained in this crate
///
/// Eiter a single struct or a list of structs can be provided.
/// `StructMergeRef<T>` will then be implemented on each given target struct.
///
/// Examples:
/// - `#[struct_merge_ref(crate::structs::Base)]`
/// - `#[struct_merge_ref([crate::structs::Base, crate:structs::Other])]`
///
/// `struct.rs`
/// ```ignore
/// use struct_merge::struct_merge_ref;
///
/// pub struct Base {
///     pub test: String,
/// }
///
/// pub struct Other {
///     pub test: String,
/// }
///
/// #[struct_merge_ref([crate::structs::Base, crate:structs::Other])]
/// pub struct Test {
///     pub test: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn struct_merge_ref(args: TokenStream, struct_ast: TokenStream) -> TokenStream {
    struct_merge_base(args, struct_ast, Mode::Borrowed)
}

/// This enum is used to differentiate between owned and borrowed merge behavior.
/// Depending on this, we need to generate another trait impl and slightly different code.
enum Mode {
    Owned,
    Borrowed,
}

fn struct_merge_base(args: TokenStream, mut struct_ast: TokenStream, mode: Mode) -> TokenStream {
    let parsed_args = parse_macro_input!(args as Expr);
    // Check if we can find the src root path of this crate.
    // Return early if it doesn't exist.
    let src_root_path = match get_root_src_path(&parsed_args) {
        Some(path) => path,
        None => return struct_ast,
    };

    // Parse the main macro input as a struct.
    // We work on a clone of the struct ast.
    // That way we don't have to parse it lateron when we return it.
    let cloned_struct_ast = struct_ast.clone();
    let src_struct = parse_macro_input!(cloned_struct_ast as ItemStruct);

    // Get the input paths from the given argument expressions.
    let paths = parse_input_paths(parsed_args);

    // Go through all paths and process the respective struct.
    let mut impls = Vec::new();
    for path in paths {
        // Make sure we found the struct at that path.
        let dest_struct = match get_struct_from_path(src_root_path.clone(), path.clone()) {
            Ok(ast) => ast,
            Err(error) => {
                impls.push(error);
                continue;
            }
        };

        // Generate the MergeStruct trait implementations.
        match generate_impl(
            &mode,
            src_struct.ident.clone(),
            path,
            src_struct.clone(),
            dest_struct,
        ) {
            Ok(ast) => impls.push(ast),
            Err(error) => {
                impls.push(error);
                continue;
            }
        }
    }

    // Merge all generated pieces of the code with the original unaltered struct.
    struct_ast.extend(impls.into_iter().map(TokenStream::from));

    // Hand the final output tokens back to the compiler.
    struct_ast
}
