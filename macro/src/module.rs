use std::path::PathBuf;

use syn::{spanned::Spanned, ExprPath, Ident, Item, ItemStruct, Token};

/// This function takes a path to a struct and returns the AST of that struct.
///
/// There is no easy way to do module resolution during this stage of the compilation.
/// Some non-trivial limitations:
/// - Non-public structs. I.e. structs that aren't fully internally visible.
/// - Alternating mix of `mod {}` code blocks and actual file modules.
/// - Code isn't located in `src` folder. We have to parse the `Cargo.toml` to resolve this.
///
/// There are many path resolution methods not implemented.
/// - [x] Import path is equivalent to file path.
/// - [ ] struct in (potentially nested or alterating) `mod {}` block in file.
/// - [ ] Struct located at root of crate. E.g. `lib.rs`.
/// - [ ] Src root dir isn't `src'.
/// - [ ] Struct is located in integration tests.
pub fn get_struct_from_path(mut file_path: PathBuf, path: ExprPath) -> Option<ItemStruct> {
    // Start searching for files from the project root.
    let path_span = path.span();

    let mut segments = path.path.segments.into_iter().peekable();
    // Make sure the root of the path is the current crate.
    let first = segments.next().unwrap();
    let crate_token = Token![crate](first.span());
    let crate_ident = Ident::from(crate_token);
    if first.ident != crate_ident {
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
        if segments.peek().is_none() {
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
                return None;
            }

            // TODO: This breaks if there are non-file modules.
            // A much better and more dynamic module resolution is needed for that to work.
        }
    };

    // Read and parse the file.
    let file_content = ok_or_err_return!(
        std::fs::read_to_string(&file_path),
        path_span,
        "Failed to open file: {:?}"
    );

    let file_ast = ok_or_err_return!(
        syn::parse_file(&file_content),
        path_span,
        "Failed to parse file: {:?}"
    );

    for item in file_ast.items.into_iter() {
        if let Item::Struct(item_struct) = item {
            if item_struct.ident == target_struct_name {
                return Some(item_struct);
            }
        }
    }

    err!(
        path_span,
        "Didn't find struct {} in file {:?}",
        target_struct_name,
        &file_path
    );

    None
}
