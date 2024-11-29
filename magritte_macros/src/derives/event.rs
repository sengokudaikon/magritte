use crate::derives::attributes::{split_generics, Table, Event, resolve_table_name};
use deluxe::ExtractAttributes;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

pub fn expand_derive_event(mut input: DeriveInput, parent_input: Option<DeriveInput>) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = split_generics(&input);
    let table_attr = Table::extract_attributes(&mut input.attrs)?;
    
    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Event can only be derived for enums",
            ));
        }
    };
    let parent_ident = if let Some(parent_input) = parent_input {
        parent_input.ident
    } else {
        format_ident!("event")
    };
    let parent = quote!(#parent_ident);
    // Get Table name from parent struct's table attribute
    let table_name = resolve_table_name(&table_attr, &parent_ident);
    let table_name_str = &*table_name;
    let err_type = quote!(magritte::prelude::EventFromStrErr);

    let mut variants = Vec::new();
    let mut defs = Vec::new();
    let mut names = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let attrs = Event::extract_attributes(&mut variant.attrs.clone())?;

        let event_name = attrs
            .name
            .clone()
            .unwrap_or_else(|| variant_name.to_string());

        let when = attrs.when.as_ref().ok_or_else(|| {
            syn::Error::new_spanned(variant, "Event must specify 'when' condition")
        })?;
        let when_str = quote!(#when).to_string();

        let then = attrs
            .then
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(variant, "Event must specify 'then' action"))?;
        let then_str = quote!(#then).to_string();

        let comment = match attrs.comment {
            Some(c) => quote!(Some(#c.to_string())),
            None => quote!(None),
        };
        let overwrite = attrs.overwrite;
        let if_not_exists = attrs.if_not_exists;

        let def = quote! {
            #ident::#variant_name => EventDef::new(
                #event_name,
                #table_name,
                #when_str,
                #then_str,
                #comment,
                #overwrite,
                #if_not_exists
            )
        };

        variants.push(quote!(#variant_name));
        defs.push(def);
        names.push(event_name);
    }

    // Generate the enum and all implementations separately
    let enum_def = quote! {
        #[derive(Debug, Copy, Clone, strum::EnumIter)]
        #[derive(PartialEq, Eq)]
        pub enum #ident #type_generics #where_clause {
            #(#variants,)*
            #[doc(hidden)]
            __Phantom(::std::marker::PhantomData<#parent #type_generics>)
        }
    };

    let trait_impls = quote! {
        #[automatically_derived]
        impl #impl_generics EventTrait for #ident #type_generics #where_clause {
            type EntityName = #parent #type_generics;

            fn def(&self) -> EventDef {
                match self {
                    #(#defs,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }

        impl #impl_generics EventType for #ident #type_generics #where_clause {
            fn table_name() -> &'static str {
                #table_name_str
            }

            fn event_name(&self) -> & str {
                match self {
                    #(#ident::#variants => &*#names,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }

        impl #impl_generics std::str::FromStr for #ident #type_generics #where_clause {
            type Err = #err_type;

            fn from_str(s: &str) -> Result<#ident, #err_type> {
                match s {
                    #(s if s == &*#names => Ok(#ident::#variants),)*
                    _ => Err(<#err_type>::new(s.to_owned()))
                }
            }
        }

        impl #impl_generics AsRef<str> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                match self {
                    #(#ident::#variants => &*#names,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }

        impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#ident::#variants => write!(f, "{}", #names),)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }
    };

    Ok(quote! {
        #enum_def

        #trait_impls
    })
}
