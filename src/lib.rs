#![feature(proc_macro_diagnostic)]

use module::get_struct_from_path;
use path::{get_root_src_path, parse_input_paths};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ItemStruct};

macro_rules! err {
    ($span:ident, $($text:expr),*) => {
        $span.span()
            .unwrap()
            .error(format!($($text,)*))
            .emit();
    }
}

/// Takes a statement returning a Result and emits a compiler error on the given Span if an error
/// occurred. Otherwise, return the value of the successful statement.
macro_rules! ok_or_err_return {
    ($expr:expr, $span:ident, $($text:expr),*) => {
        match $expr {
            Ok(result) => result,
            Err(error) =>  {
                err!($span, $($text,)* error);
                return None;
            }
        }
    }
}

mod module;
mod path;

#[proc_macro_attribute]
pub fn struct_merge(args: TokenStream, mut struct_ast: TokenStream) -> TokenStream {
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
    let merge_struct = parse_macro_input!(cloned_struct_ast as ItemStruct);

    // Get the input paths from the given argument expressions.
    let paths = parse_input_paths(parsed_args);

    let mut impls = Vec::new();
    // Go through all paths and process the respective struct.
    for path in paths.clone() {
        // Make sure we found the struct in that file.
        let target_struct_ast = match get_struct_from_path(src_root_path.clone(), path) {
            Some(ast) => ast,
            None => continue,
        };
    }

    let name = merge_struct.ident;
    let expanded = quote! {
        // The generated impl.
        impl #name {
            pub fn merge_into(&self) {
                println!("This is a test:");
            }
        }
    };
    impls.push(TokenStream::from(expanded));

    // Merge all generated pieces of the code with the original unaltered struct.
    struct_ast.extend(impls);

    // Hand the final output tokens back to the compiler.
    struct_ast
}
