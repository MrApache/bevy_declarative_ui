use bevy_declarative_ui_parser::LayoutReader;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_str, DeriveInput, LitStr, Type};

#[proc_macro_attribute]
pub fn ui_layout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let plugin_type: Type = parse_str("bevy::prelude::Plugin").unwrap();
    let app_type: Type = parse_str("bevy::app::App").unwrap();


    let file = parse_macro_input!(attr as LitStr).value();
    let input = parse_macro_input!(item as DeriveInput);

    let content = std::fs::read_to_string(file.as_str()).unwrap();
    let mut reader = LayoutReader::new(content.as_str(), file.as_str());
    let result = reader.parse().unwrap();


    let struct_name = input.ident;
    let vis = input.vis;

    let expanded = quote! {
        #vis struct #struct_name;
        
        #vis impl #plugin_type for #struct_name {
            fn build(&self, app: &mut #app_type) {

            }
        }
    };
    
    TokenStream::from(expanded)
}


