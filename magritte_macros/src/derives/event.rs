use crate::derives::attributes::{split_generics, Event};
use deluxe::ExtractAttributes;
use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{Data, DeriveInput};

fn strip_events_suffix(ident: &syn::Ident) -> String {
    let name = ident.to_string();
    if name.ends_with("Events") {
        name[..name.len() - 6].to_string()
    } else {
        name
    }
}

pub fn expand_derive_event(mut input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = split_generics(&input);
    
    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Event can only be derived for enums",
            ));
        }
    };
    let err_type = quote!(magritte::prelude::EventFromStrErr);

    let mut variants = Vec::new();
    let mut defs = Vec::new();
    let mut names = Vec::new();
    let mut table_name = None;

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let attrs = Event::extract_attributes(&mut variant.attrs.clone())?;
        
        // Get and validate table name from attributes
        let current_table = attrs.table.as_ref().ok_or_else(|| {
            syn::Error::new_spanned(variant, "Event must specify 'table' attribute")
        })?;

        // Ensure all variants reference the same table
        if let Some(ref prev_table) = table_name {
            if prev_table != current_table {
                return Err(syn::Error::new_spanned(
                    variant,
                    "All events in an enum must reference the same table",
                ));
            }
        } else {
            table_name = Some(current_table.clone());
        }
        
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
                #current_table,
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

    let table_name = table_name.expect("Table name must be specified");
    // Get parent struct name by stripping "Events" from enum name
    let parent_struct_name = strip_events_suffix(ident);
    let parent = format_ident!("{}", parent_struct_name);

    // Generate the enum and all implementations separately

    let trait_impls = quote! {

        impl #impl_generics #parent #type_generics #where_clause {
            pub fn events() -> impl Iterator<Item = #ident #type_generics> {
                use strum::IntoEnumIterator;
                #ident::iter()
            }
        }

        #[automatically_derived]
        impl #impl_generics magritte::prelude::EventTrait for #ident #type_generics #where_clause {
            type EntityName = #parent #type_generics;

            fn def(&self) -> magritte::prelude::EventDef {
                match self {
                    #(#defs,)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics magritte::prelude::EventType for #ident #type_generics #where_clause {
            fn table_name() -> &'static str {
                #table_name
            }

            fn event_name(&self) -> &str {
                match self {
                    #(#ident::#variants => #names,)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics std::str::FromStr for #ident #type_generics #where_clause {
            type Err = #err_type;

            fn from_str(s: &str) -> Result<#ident, #err_type> {
                match s {
                    #(s if s == #names => Ok(#ident::#variants),)*
                    _ => Err(<#err_type>::new(s.to_owned()))
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics core::convert::AsRef<str> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                match self {
                    #(#ident::#variants => #names,)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#ident::#variants => write!(f, "{}", #names),)*
                }
            }
        }
    };

    Ok(quote! {

        #trait_impls
    })
}
