use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn struct_merge(
    attr: proc_macro::TokenStream,
    struct_ast: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let temp_ast = struct_ast.clone();
    let input = parse_macro_input!(temp_ast as ItemStruct);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    let token_stream = attr.to_string();

    let expanded = quote! {
        // The generated impl.
        impl #name {
            pub fn merge_into(&self) {
                println!("This is a test: {:?}", #token_stream);
            }
        }
    };

    // Hand the output tokens back to the compiler.
    let mut final_stream = proc_macro::TokenStream::from(struct_ast);
    final_stream.extend(vec![proc_macro::TokenStream::from(expanded)]);

    final_stream
}
