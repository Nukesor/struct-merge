#![feature(proc_macro_diagnostic)]

use std::{fs::File, path::PathBuf};

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Expr, ExprPath, Ident, Item, ItemStruct, Token};

/// Takes a statement returning a Result and emits a compiler error on the given Span if an error
/// occurred. Otherwise, return the value of the successful statement.
macro_rules! ok_or_continue_err {
    ($expr:expr, $span:ident, $($text:expr),*) => {
        match $expr {
            Ok(result) => result,
            Err(error) =>  {
                $span.unwrap()
                    .error(format!($($text,)* error))
                    .emit();
                continue;
            }
        }
    }
}

macro_rules! err {
    ($span:ident, $($text:expr),*) => {
        $span.span()
            .unwrap()
            .error(format!($($text,)*))
            .emit();
    }
}

#[proc_macro_attribute]
pub fn struct_merge(args: TokenStream, mut struct_ast: TokenStream) -> TokenStream {
    let parsed_args = parse_macro_input!(args as Expr);

    let src_root_path = match std::env::var("CARGO_MANIFEST_DIR") {
        Err(error) => {
            err!(
                parsed_args,
                "Couldn't read CARGO_MANIFEST_DIR environment variable: {}",
                error
            );

            // Exit early, as there's nothing we can do if the path doesn't exist.
            return struct_ast;
        }
        Ok(path) => {
            let mut path = PathBuf::from(path);
            if !path.exists() {
                err!(
                    parsed_args,
                    "CARGO_MANIFEST_DIR path doesn't exist: {:?}",
                    path
                );

                // Exit early, as there's nothing we can do if the path doesn't exist.
                return struct_ast;
            }

            // TODO: We expect the source tree to start in `$CARGO_MANIFEST_DIR/src`.
            // For everything else, we would have to manually parse the Cargo manifest.
            path.push("src");

            path
        }
    };

    // Work on a clone of the struct ast.
    // That way we don't have to parse it lateron again.
    let cloned_struct_ast = struct_ast.clone();
    // Parse the main macro input as a struct.
    let merge_struct = parse_macro_input!(cloned_struct_ast as ItemStruct);

    // Get the input paths from the given argument expressions.
    let paths = parse_input_paths(parsed_args);

    // Go through all paths and process the respective struct.
    'outer: for path in paths.clone() {
        // Start searching for files from the project root.
        let mut file_path = src_root_path.clone();
        let path_span = path.span().clone();

        let mut segments = path.path.segments.into_iter().peekable();
        // Make sure the root of the path is the current crate.
        let first = segments.next().unwrap();
        if first.ident != Ident::from(Token![crate](first.span())) {
            err!(
                first,
                "struct_merge only supports paths in the current 'crate::' space for now."
            );
        }

        // Get the file path for the specified Rust path.
        let target_struct_name = loop {
            // We know that the next value exists.
            // If no further value exists, we break and exit early.
            let segment = segments.next().unwrap();

            // The last identifier is the the name of the struct.
            // Break, so it doen't get added to the path.
            if let None = segments.peek() {
                break segment.ident;
            }

            // Push the next identifier to the path.
            file_path.push(segment.ident.to_string());

            // Check if we find a folder for that module.
            if !file_path.exists() {
                // In case we couldn't find a folder, try a Rust file.
                // Set the extension for rust source code files.
                file_path.set_extension("rs");

                if !file_path.exists() {
                    err!(segment, "Cannot find file for path: {:?}", file_path);
                    continue 'outer;
                }

                // TODO: This breaks if there are non-file modules.
                // A much better and more dynamic module resolution is needed for that to work.
            }
        };

        // Read and parse the file.
        let file_content = ok_or_continue_err!(
            std::fs::read_to_string(&file_path),
            path_span,
            "Failed to open file: {:?}"
        );

        let file_ast = ok_or_continue_err!(
            syn::parse_file(&file_content),
            path_span,
            "Failed to parse file: {:?}"
        );

        let mut target_struct_ast = None;

        for item in file_ast.items.into_iter() {
            if let Item::Struct(item_struct) = item {
                if item_struct.ident == target_struct_name {
                    target_struct_ast = Some(item_struct);
                }
            }
        }

        // Make sure we found the struct in that file.
        let target_struct_ast = if let Some(ast) = target_struct_ast {
            ast
        } else {
            err!(
                path_span,
                "Didn't find struct {} in file {:?}",
                target_struct_name,
                &file_path
            );
            continue;
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

    // Hand the output tokens back to the compiler.
    struct_ast.extend(vec![TokenStream::from(expanded)]);

    struct_ast
}

/// Extract the input paths from the macro arguments.
///
/// Both, a single path and an array of paths is supported.
/// I.e.
/// - `merge_struct(crate::some::path)`
/// - `merge_struct([crate::some::struct, crate::some_other::struct])`
fn parse_input_paths(args: Expr) -> Vec<ExprPath> {
    match args {
        Expr::Path(path) => {
            vec![path]
        }
        Expr::Array(array) => {
            let mut paths = vec![];
            for expr in array.elems {
                match expr {
                    Expr::Path(path) => paths.push(path),
                    _ => {
                        err!(expr, "Only paths are allowed in struct_merge's attribute.");
                    }
                }
            }
            paths
        }
        _ => {
            err!(
                args,
                "struct_merge's macro parameters should be either a single path {} ",
                "or a vector of paths, such as '[crate::your::path]'."
            );

            return vec![];
        }
    }
}
