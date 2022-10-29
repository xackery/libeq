use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Field, Fields, GenericArgument, Ident,
    PathArguments, Type, TypePath,
};

#[proc_macro_derive(FragParser, attributes(fragment_id, present_when, count))]
pub fn fragment_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let steps = parse_steps(&input.data);
    let fields = build_frag(&input.data);

    let struct_ident = &input.ident;
    let struct_name = &input.ident.to_string();
    let frag_id = get_fragment_id_attribute(&input.attrs);

    quote! {
        use crate::parser::{WResult, FragmentParser};

        impl FragmentParser for #struct_ident {
            type T = Self;

            const TYPE_ID: u32 = #frag_id;
            const TYPE_NAME: &'static str = #struct_name;

            fn parse(i: &[u8]) -> WResult<#struct_ident> {
                #steps

                Ok((i, #fields))
            }
        }
    }
    .into()
}

fn build_frag(data: &Data) -> TokenStream {
    let field_names = match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields.named.iter().map(|f| &f.ident),
            Fields::Unnamed(ref _fields) => unimplemented!(),
            Fields::Unit => unimplemented!(),
        },
        Data::Enum(ref _data) => {
            unimplemented!()
        }
        Data::Union(_) => unimplemented!(),
    };

    quote! {
        Self {
            #(#field_names),*
        }
    }
    .into()
}

fn parse_steps(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields
                .named
                .iter()
                .map(|f| parser_for_field(f))
                .collect::<TokenStream>(),
            Fields::Unnamed(ref _fields) => unimplemented!(),
            Fields::Unit => unimplemented!(),
        },
        Data::Enum(ref _data) => {
            unimplemented!()
        }
        Data::Union(_) => unimplemented!(),
    }
    .into()
}

fn get_inner_parser_function(path_args: &PathArguments, attrs: &Vec<Attribute>) -> TokenStream {
    match path_args {
        PathArguments::AngleBracketed(args) => match args.args.last().unwrap() {
            GenericArgument::Type(inner_type) => match inner_type {
                Type::Path(inner_path) => get_parser_function(inner_path, attrs),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn get_parser_function(path: &TypePath, attrs: &Vec<Attribute>) -> TokenStream {
    let path_last = path.path.segments.last().unwrap();
    let span = path_last.ident.span();
    match path_last.ident.to_string().as_str() {
        "Option" => {
            let presence_condition = get_presence_attribute(attrs);
            let inner = get_inner_parser_function(&path_last.arguments, attrs);
            quote_spanned! { span =>
                (|input| {
                    if #presence_condition {
                        nom::combinator::map(#inner, |x| Some(x))(input)
                    } else {
                        Ok((input, None))
                    }
                })
            }
        }
        "Vec" => {
            let count = get_count_attribute(attrs);
            let inner = get_inner_parser_function(&path_last.arguments, attrs);
            quote_spanned! { span => nom::multi::count(#inner, #count as usize) }
        }
        "u8" => {
            quote_spanned! { span => nom::number::complete::le_u8 }
        }
        "i8" => {
            quote_spanned! { span => nom::number::complete::le_i8 }
        }
        "u16" => {
            quote_spanned! { span => nom::number::complete::le_u16 }
        }
        "i16" => {
            quote_spanned! { span => nom::number::complete::le_i16 }
        }
        "u32" => {
            quote_spanned! { span => nom::number::complete::le_u32 }
        }
        "i32" => {
            quote_spanned! { span => nom::number::complete::le_i32 }
        }
        "f32" => {
            quote_spanned! { span => nom::number::complete::le_f32 }
        }
        _ => {
            quote_spanned! { span => #path::parse }
        }
    }
}

fn parser_for_field(field: &Field) -> TokenStream {
    let name = field.ident.as_ref().unwrap();
    let attrs = &field.attrs;
    match &field.ty {
        Type::Path(type_path) => {
            let parser_function = get_parser_function(type_path, attrs);
            quote_spanned! { name.span() =>
                let (i, #name) = #parser_function(i)?;
            }
        }
        _ => {
            unimplemented!()
        }
    }
    .into()
}

//////////////////////////////
// Attributes
//////////////////////////////

fn get_attr_by_name<'a>(attrs: &'a Vec<Attribute>, name: &str) -> Option<&'a Attribute> {
    attrs.iter().find(|attr| {
        attr.path
            .segments
            .last()
            .map(|seg| seg.ident.to_string().as_str() == name)
            .unwrap_or(false)
    })
}

fn get_fragment_id_attribute(attrs: &Vec<Attribute>) -> TokenStream {
    get_attr_by_name(&attrs, "fragment_id").map_or(quote! { 0x0 }, |attr| {
        attr.clone().tokens.into_iter().last().unwrap().into()
    })
}

fn get_count_attribute(attrs: &Vec<Attribute>) -> TokenStream {
    get_attr_by_name(attrs, "count")
        .map_or_else(
            || panic!("count attribute required!"),
            |attr| match attr.clone().tokens.into_iter().last().unwrap() {
                TokenTree::Group(g) => g.stream(),
                _ => panic!("Parens required!"),
            },
        )
        .into()
}

fn get_presence_attribute(attrs: &Vec<Attribute>) -> TokenStream {
    get_attr_by_name(attrs, "present_when")
        .map_or_else(
            || panic!("present_when attribute required!"),
            |attr| match attr.clone().tokens.into_iter().last().unwrap() {
                TokenTree::Group(g) => g.stream(),
                _ => panic!("Parens required!"),
            },
        )
        .into()
}

//struct CountParams(syn::Lifetime, syn::Ident, syn::Ident);
//impl syn::Parse for MyParams {
//    fn parse(input: syn::ParseStream) -> Result<Self> {
//        let content;
//        syn::parenthesized!(content in input);
//        let lifetime = content.parse()?;
//        content.parse::<Token![,]>()?;
//        let type1 = content.parse()?;
//        content.parse::<Token![,]>()?;
//        let type2 = content.parse()?;
//        Ok(MyParams(lifetime, type1, type2))
//    }
//}
