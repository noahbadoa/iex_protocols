mod struct_parsing;
mod derives;

use quote::ToTokens;

#[proc_macro]
pub fn parse_struct(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_input = syn::parse_macro_input!(item as struct_parsing::DervivedStruct);
    parsed_input.to_token_stream().into()
}

#[proc_macro]
pub fn parse_enum(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_input = syn::parse_macro_input!(item as struct_parsing::NestedEnum);
    parsed_input.to_token_stream().into()
}

#[proc_macro]
pub fn parse_deep(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_input = syn::parse_macro_input!(item as derives::DeepPlus);
    parsed_input.to_token_stream().into()
}