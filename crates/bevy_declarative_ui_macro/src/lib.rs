use std::path::PathBuf;

use bevy_declarative_ui_parser::LayoutReader;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, DeriveInput, LitStr, Type};

macro_rules! try_unwrap {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                let error_msg = err.to_string();
                return quote! {
                    compile_error!(#error_msg);
                }.into();
            }
        }
    };
}

#[proc_macro_attribute]
pub fn ui_layout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let plugin_type: Type = parse_str("bevy::prelude::Plugin").unwrap();
    let app_type: Type = parse_str("bevy::app::App").unwrap();

    let relative_path = parse_macro_input!(attr as LitStr).value();

    let absolute_path = try_unwrap!(to_absolute_path(&relative_path));
    let file_content  = try_unwrap!(std::fs::read_to_string(&absolute_path));
    let _parsed_layout = try_unwrap!(LayoutReader::new(&file_content, &absolute_path).parse());

    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = input.ident;
    let vis = input.vis;

    let expanded = quote! {
        #vis struct #struct_name;
        
        impl #plugin_type for #struct_name {
            fn build(&self, app: &mut #app_type) {

            }
        }
    };
    
    TokenStream::from(expanded)
}

fn to_absolute_path(relative_path: &str) -> Result<String, TokenStream> {
    let project_root = std::env::var("CARGO_MANIFEST_DIR");
    if let Err(error) = project_root {
        let error_msg = error.to_string();
        return Err(quote!(compile_error!(#error_msg)).into())
    }
 
    let mut path = PathBuf::from(project_root.unwrap());
    path.push(relative_path);
    Ok(path.to_string_lossy().replace('\\', "\\\\"))
}
