use quote::{ToTokens, quote};
use syn::{Token, parse::Parse};

type MemberNameType = syn::Ident;

pub struct NestedVariant{
    pub name : MemberNameType,
    pub value : syn::Expr,
}

impl Parse for NestedVariant{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = MemberNameType::parse(input)?;
        _ = input.parse::<Token![=]>()?;
        let value = syn::Expr::parse(input)?;

        Ok(Self { name, value })
    }
}

impl ToTokens for NestedVariant{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let NestedVariant { name, value } = &self;

        let stream = quote! {
            #name = #value
        };

        tokens.extend(stream);
    }
}

pub struct NestedEnum{
    pub name : MemberNameType,
    pub varaints : Vec<NestedVariant>
}

impl ToTokens for NestedEnum{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let NestedEnum { name, varaints } = &self;
        let varaint_name = varaints.iter().map(|x|{x.name.clone()});

        let output = quote! {
            #[repr(u8)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #name{
                #(#varaints,)*
            }

            impl #name{
                pub const fn is_vaild(byte : [u8; 1]) -> bool{
                    #((Self::#varaint_name as u8 == byte[0]) |)* false
                }
            }
        };

        tokens.extend(output);
    }
}

impl Parse for NestedEnum{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<Token![enum]>()?;
        let name = input.parse::<MemberNameType>()?;

        let content;
        syn::braced!(content in input);

        let varaints = content.parse_terminated(NestedVariant::parse, Token![,])?;
        let varaints = varaints.into_iter().collect::<Vec<_>>();

        let out = Self{name, varaints};

        Ok(out)
    }
}

pub enum LittleEndianType{
    Path(syn::Path),
    Qualified(&'static str),
}

impl Parse for  LittleEndianType{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path = input.parse::<syn::Path>()?;

        let ident = path.get_ident();
        if ident.is_none() {return Ok(Self::Path(path));}
        let ident = ident.unwrap();
        let ident_string = ident.to_string();

        // couldn't figure out syn generics
        let ident = match ident_string.as_str() {
            "i8" => {"Little::<i8>"},
            "u8" => {"Little::<u8>"},

            "i16" => {"Little::<i16>"},
            "u16" => {"Little::<u16>"},

            "i32" => {"Little::<i32>"},
            "u32" => {"Little::<u32>"},

            "i64" => {"Little::<i64>"},
            "u64" => {"Little::<u64>"},

            "i128" => {"Little::<i128>"},
            "u128" => {"Little::<u128>"},
            
            _ => {return Ok(Self::Path(path));}
        };

        Ok(Self::Qualified(ident))
    }
}

impl ToTokens for LittleEndianType{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let stream = match self {
            Self::Path(path) => {
                quote! {#path}
            }
            Self::Qualified(qualifed) => {
                qualifed.parse().unwrap()
            }
        };

        tokens.extend(stream);
    }
}

pub enum MemberType{
    External(LittleEndianType),
    Enum(NestedEnum),

}

impl Parse for MemberType{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let out = if input.peek(Token![enum]){
            MemberType::Enum(input.parse::<NestedEnum>()?)
        }else{
            MemberType::External(input.parse::<LittleEndianType>()?)
        };

        Ok(out)
    }   
}

impl ToTokens for MemberType{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let out =match self {
            Self::Enum(nested) => {
                let NestedEnum { name, varaints : _ } = nested;
                quote! {#name}
            }
            
            Self::External(external) => {
                quote! {#external}
            }
        };

        tokens.extend(out);
    }
}

pub struct Member{
    pub visibility : syn::Visibility,
    pub name : MemberNameType,
    pub member_type : MemberType,
}

impl Parse for Member{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let visibility = syn::Visibility::parse(input)?;
        let name = MemberNameType::parse(input)?;
        _ = input.parse::<Token![:]>()?;
        let member_type = MemberType::parse(input)?;

        Ok(Self { visibility, name, member_type })
    }
}

impl ToTokens for Member{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Member { visibility, name, member_type } = &self;

        let output = quote! {
            #visibility #name : #member_type
        };

        tokens.extend(output);
    }
}

pub struct DervivedStruct{
    pub visibility: syn::Visibility,
    pub name : MemberNameType,
    pub message_type : syn::Expr,
    pub fields : Vec<Member>,
}

impl DervivedStruct{
    fn member_enums(&self, tokens: &mut proc_macro2::TokenStream){
        for field in &self.fields{
            if let MemberType::Enum(ref inner) = field.member_type{
                inner.to_tokens(tokens);
            }
        }
    }

    fn definition(&self, tokens: &mut proc_macro2::TokenStream){
        let DervivedStruct { visibility, name, fields, message_type : _ } = &self;

        let struct_definition = quote! {
            #[repr(C, packed)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #visibility struct #name{
                #(#fields,)*
            }
        };

    

        tokens.extend(struct_definition);
    }


    fn dervied_methods(&self, tokens: &mut proc_macro2::TokenStream, needs_validation : &[syn::Path]){
        let DervivedStruct { visibility:_, name, fields:_ , message_type} = &self;

        let filtered_fields = self.fields.iter().filter_map(|x|{
            let out = match &x.member_type {
                MemberType::Enum(nested) => {
                    let field_name = &x.name;
                    let enum_name = &nested.name;

                    quote! {
                        #enum_name::is_vaild([bytes[core::mem::offset_of!(#name, #field_name)]])
                    }
                }

                MemberType::External(external) => {
                    if let LittleEndianType::Path(path) = external{
                        if !needs_validation.contains(path) {return core::option::Option::None;}
                    }else{
                        return core::option::Option::None;
                    }

                    let field_name = &x.name;

                    quote! {
                        #external::is_vaild(unsafe{
                            let ptr : *const u8 = bytes.as_ptr().add(core::mem::offset_of!(#name, #field_name));
                            core::mem::transmute(ptr)
                        })
                    }
                }
            };

            Some(out)
        });

        let methods = quote! {
            impl #name{
                pub const MESSAGE_TYPE : u8 = #message_type;
                pub const fn is_vaild(bytes : &[u8; core::mem::size_of::<Self>()]) -> bool {
                    #(#filtered_fields &&)* true
                }
            }

        };

        tokens.extend(methods);
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream, needs_validation : &[syn::Path]) {
        self.member_enums(tokens);
        self.definition(tokens);
        self.dervied_methods(tokens, needs_validation);
    }   

    pub fn to_quote<'a>(&'a self, vaildate : &'a [syn::Path]) -> DervivedStructQuote<'a>{
        DervivedStructQuote{inner : self, vaildate}
    }
}

impl ToTokens for DervivedStruct{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.to_quote(&[]).to_tokens(tokens)
    }
}

pub struct DervivedStructQuote<'a>{
    pub inner : &'a DervivedStruct,
    pub vaildate : &'a [syn::Path]
}

impl<'a> ToTokens for DervivedStructQuote<'a>{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.inner.to_tokens(tokens, self.vaildate);
    }
}


impl Parse for DervivedStruct{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let visibility = syn::Visibility::parse(input)?;
        let _ = input.parse::<Token![struct]>();
        let name = input.parse::<MemberNameType>()?;
        let message_type = input.parse::<syn::Expr>()?;

        let content;
        syn::braced!(content in input);

        let content = content.parse_terminated(Member::parse, Token![,])?;
        let fields = content.into_iter().collect::<Vec<_>>();

        let out = Self{visibility, name, fields, message_type};

        Ok(out)
    }
}

#[test]
fn struct_macro_test(){
    use core::str::FromStr;

    let stream = proc_macro2::TokenStream::from_str("
        pub struct AddOrders 4321 {
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

    let out = syn::parse2::<DervivedStruct>(stream).unwrap();

    let mut output_stream = proc_macro2::TokenStream::new();
    out.to_tokens(&mut output_stream, &[]);
    let output_readable = output_stream.to_string();
    println!("{:?}", output_readable);
}
