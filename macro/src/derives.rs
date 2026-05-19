use quote::{ToTokens, quote};
use syn::parse::Parse;
use crate::struct_parsing::DervivedStruct;




pub struct DeepPlus{
    vaildate : Vec<syn::Path>,
    structs : Vec<DervivedStruct>,
}

impl DeepPlus{
    fn parse_validation(input: &syn::parse::ParseStream) -> syn::Result<Vec<syn::Path>> {
        
        let content;
        syn::braced!(content in input);

        let varaints = content.parse_terminated(syn::Path::parse, syn::Token![,])?;
        let needs_validation = varaints.into_iter().collect::<Vec<_>>();

        Ok(needs_validation)
    }

    fn parse_structs(input: &syn::parse::ParseStream) -> syn::Result<Vec<DervivedStruct>>{
        let mut structs: Vec<DervivedStruct> = Vec::new();
        while !input.is_empty() {
            let next = input.parse::<DervivedStruct>()?;
            structs.push(next);
        }

        Ok(structs)
    }
}

impl Parse for DeepPlus{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vaildate = Self::parse_validation(&input)?;
        let structs = Self::parse_structs(&input)?;

        Ok(Self { vaildate, structs })
    }
}

pub struct IsRef(pub bool);
impl ToTokens for IsRef{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.0{
            tokens.extend( quote! {&});
        }
    }   
}

pub struct IsLifetime<const NAME : char>(pub bool);
impl<const NAME : char> ToTokens for IsLifetime<NAME>{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let lifetime = syn::Lifetime::new(format!("'{NAME}").as_str(), proc_macro2::Span::call_site());

        if self.0{
            tokens.extend( quote! {#lifetime});
        }
    }   
}

impl DeepPlus{
    fn definition_tokens(&self, tokens: &mut proc_macro2::TokenStream){
        let DeepPlus { structs, vaildate } = &self;
        let structs = structs.iter().map(|x|{x.to_quote(vaildate)});

        let stream = quote! {#(#structs)*};
        tokens.extend(stream);
    }

    fn global_token(&self, tokens: &mut proc_macro2::TokenStream, enum_name : &syn::Ident, is_ref : bool) {
        let lifetime = IsLifetime::<'a'>(is_ref);
        let refernce = IsRef(is_ref);
        let struct_name = self.structs.iter().map(|x|{&x.name});

        let out = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #enum_name <#lifetime>{
                #(#struct_name(#refernce #lifetime #struct_name),)*
            }
        };

        tokens.extend(out);
    }

    fn token_parsing(&self, tokens: &mut proc_macro2::TokenStream, reference_name : &syn::Ident, message_type_enum : &syn::Ident) {
        let field_name = self.structs.iter().map(|x|{&x.name});

        let output = quote! {
            impl<'a> #reference_name<'a>{
                pub const fn parse_packet(bytes : &[u8]) -> core::result::Result::<#reference_name<'a>, MessageParseError>{

                    if bytes.len() == 0 {return Err(MessageParseError::Invalid);}

                    let message_type = #message_type_enum::from_byte([bytes[0]]);
                    let message_type = match message_type{
                        None => {
                            return Err(MessageParseError::Unknown);
                        }
                        Some(message_type) => message_type
                    };

                    let field_ptr = unsafe{bytes.as_ptr().add(1)};

                    let success = match message_type{
                        #(
                            #message_type_enum::#field_name => {
                                if core::mem::size_of::<#field_name>() > (bytes.len() - 1) {return Err(MessageParseError::Invalid);}

                                let exact_sized = unsafe{core::mem::transmute(field_ptr)};
                                let vaild = #field_name::is_vaild(exact_sized);
                                if !vaild {return Err(MessageParseError::Invalid);}

                                let refernce = unsafe{core::mem::transmute(field_ptr)};
                                #reference_name::#field_name(refernce)
                            },
                        )*
                    };

                    Ok(success)
                }
            }
        };

        tokens.extend(output);
    }

    fn casting_trait(&self, tokens: &mut proc_macro2::TokenStream, reference_name : &syn::Ident, owned_name : &syn::Ident) {
        let field_name = self.structs.iter().map(|x|{&x.name});
        let field_name2 = self.structs.iter().map(|x|{&x.name});

        let output = quote! {
            impl #owned_name{
                pub const fn from_ref(reference : #reference_name) -> Self{
                    match reference{
                        #(
                            #reference_name::#field_name(inner) => {
                                #owned_name::#field_name(*inner)
                            },
                        )*
                    }
                }
            }

            impl<'a> #reference_name<'a>{
                pub const fn from_owned(owned : &'a #owned_name) -> Self{
                    match owned{
                        #(
                            #owned_name::#field_name2(inner) => {
                                #reference_name::#field_name2(inner)
                            },
                        )*
                    }
                }
            }
        };

        tokens.extend(output);
    }

    fn global_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let owned_name = syn::Ident::new("OwnedMessageEnum", proc_macro2::Span::call_site());
        let reference_name = syn::Ident::new("MessageEnum", proc_macro2::Span::call_site());
        let message_type_enum_name = syn::Ident::new("MessageType", proc_macro2::Span::call_site());


        self.global_token(tokens, &owned_name, false);
        self.global_token(tokens, &reference_name, true);

        self.message_type_enum_defintion(tokens, &message_type_enum_name);
        self.message_type_enum_match(tokens, &message_type_enum_name);

        self.token_parsing(tokens, &reference_name, &message_type_enum_name);
        self.casting_trait(tokens, &reference_name, &owned_name);
    }


    fn message_type_enum_defintion(&self, tokens: &mut proc_macro2::TokenStream, name : &syn::Ident){
        let DeepPlus { structs, vaildate:_ } = self;
        let struct_name = structs.iter().map(|x|{&x.name});
        let struct_id = structs.iter().map(|x|{&x.message_type});

        let output = quote! {
            #[repr(u8)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #name{
                #(#struct_name = #struct_id,)*
            }
        };

        tokens.extend(output);
    }

    fn message_type_enum_match(&self, tokens: &mut proc_macro2::TokenStream, name : &syn::Ident){
        let DeepPlus { structs, vaildate:_ } = self;
        let struct_name = structs.iter().map(|x|{&x.name});

        let output = quote! {
            impl #name{
                pub const fn is_vaild(byte : [u8; 1]) -> bool{
                    #(((Self::#struct_name as u8) == byte[0]) | )* false
                }

                pub const fn from_byte(byte : [u8; 1]) -> core::option::Option<Self>{
                    if !Self::is_vaild(byte) {return None;}
                    unsafe{Some(core::mem::transmute::<[u8; 1], Self>(byte))}
                } 
            }
        };

        tokens.extend(output);
    }
}

impl ToTokens for DeepPlus{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.definition_tokens(tokens);
        self.global_tokens(tokens);
    }
}

#[test]
fn deep_macro_test() {
    use core::str::FromStr;

    let stream = proc_macro2::TokenStream::from_str("
        {}
        pub struct AddOrders 1234 {
            pub side : enum Side{
                Buy = 0x38,
                Sell = 0x35,
            },
            pub timestamp: Timestamp,
            pub symbol: Symbol,
            pub order_id: OrderId,
            pub size: u32,
            pub price: Price,
        }
    ").unwrap().into();

    let out = syn::parse2::<DeepPlus>(stream).unwrap();

    let mut output_stream = proc_macro2::TokenStream::new();
    out.to_tokens(&mut output_stream);
    let output_readable = output_stream.to_string();
    println!("{:?}", output_readable);
}
