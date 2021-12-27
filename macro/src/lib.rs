#![feature(proc_macro_diagnostic)]

use generate::generate_implementations;
use module::get_struct_from_path;
use path::{get_root_src_path, parse_input_paths};
use proc_macro::TokenStream;
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

mod generate;
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
    let src_struct = parse_macro_input!(cloned_struct_ast as ItemStruct);

    // Get the input paths from the given argument expressions.
    let paths = parse_input_paths(parsed_args);

    let mut impls = Vec::new();
    // Go through all paths and process the respective struct.
    for path in paths.clone() {
        // Make sure we found the struct in that file.
        let dest_struct = match get_struct_from_path(src_root_path.clone(), path.clone()) {
            Some(ast) => ast,
            None => continue,
        };

        // Generate the MergeStruct trait implementations.
        match generate_implementations(
            src_struct.ident.clone(),
            path,
            src_struct.clone(),
            dest_struct,
        ) {
            Some(ast) => impls.push(ast),
            None => continue,
        }
    }

    // Merge all generated pieces of the code with the original unaltered struct.
    struct_ast.extend(impls);

    // Hand the final output tokens back to the compiler.
    struct_ast
}
