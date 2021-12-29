use std::path::PathBuf;

use syn::{spanned::Spanned, Expr, ExprPath};

/// Extract the input paths from the macro arguments.
///
/// Both, a single path and an array of paths is supported.
/// I.e.
/// - `merge_struct(crate::some::path)`
/// - `merge_struct([crate::some::struct, crate::some_other::struct])`
pub fn parse_input_paths(args: Expr) -> Vec<ExprPath> {
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

            vec![]
        }
    }
}

/// Get the root path of the crate that's currently using this proc macro.
/// This is done via the `CARGO_MANIFEST_DIR` variable, that's always supplied by cargo and
/// represents the directory containing the `Cargo.toml` for the current crate.
pub fn get_root_src_path(parsed_args: &Expr) -> Option<PathBuf> {
    match std::env::var("CARGO_MANIFEST_DIR") {
        Err(error) => {
            err!(
                parsed_args,
                "Couldn't read CARGO_MANIFEST_DIR environment variable: {}",
                error
            );

            // Exit early, as there's nothing we can do if the path doesn't exist.
            None
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
                return None;
            }

            // TODO: We expect the source tree to start in `$CARGO_MANIFEST_DIR/src`.
            // For everything else, we would have to manually parse the Cargo manifest.
            path.push("src");

            Some(path)
        }
    }
}
