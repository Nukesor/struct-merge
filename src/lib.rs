#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Expr, ExprPath, ItemStruct};

#[proc_macro_attribute]
pub fn struct_merge(args: TokenStream, mut struct_ast: TokenStream) -> TokenStream {
    let parsed_args = parse_macro_input!(args as Expr);
    let paths = parse_input_paths(parsed_args);

    // Work on a clone of the struct ast.
    // That way we don't have to parse it lateron again.
    let cloned_struct_ast = struct_ast.clone();
    // Parse the main macro input as a struct.
    let merge_struct = parse_macro_input!(cloned_struct_ast as ItemStruct);

    // Used in the quasi-quotation below as `#name`.
    let name = merge_struct.ident;
    let formatted_paths = format!(
        "{:?}",
        paths
            .into_iter()
            .map(|path| path.into_token_stream().to_string())
            .collect::<Vec<String>>()
    );

    let expanded = quote! {
        // The generated impl.
        impl #name {
            pub fn merge_into(&self) {
                println!("This is a test: {:?}", #formatted_paths);
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
                        expr.span()
                            .unwrap()
                            .error("Only paths are allowed in struct_merge's attribute.")
                            .emit();
                    }
                }
            }
            paths
        }
        _ => {
            args
                .span()
                .unwrap()
                .error("struct_merge's macro parameters should be either a single path or a vector of paths, such as '[crate::your::path]'.")
                .emit();

            return vec![];
        }
    }
}
