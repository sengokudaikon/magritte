use crate::derives::attributes::{split_generics, Event};
use deluxe::ExtractAttributes;
use macro_helpers::get_crate_name;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

fn strip_events_suffix(ident: &syn::Ident) -> syn::Path {
    let name = ident.to_string();
    let table_name = if name.ends_with("Events") {
        name[..name.len() - 6].to_string()
    } else {
        name
    };
    syn::parse_str::<syn::Path>(&table_name).unwrap_or_else(|_| syn::parse_quote!(#ident))
}

pub fn expand_derive_event(mut input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = split_generics(&input);
    let crate_name = get_crate_name(false);
    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Event can only be derived for enums",
            ));
        }
    };
    let err_type = quote!(#crate_name::EventFromStrErr);

    let mut variants = Vec::new();
    let mut defs = Vec::new();
    let mut names = Vec::new();
    // Get parent struct name by stripping "Events" from enum name
    let parent_struct = strip_events_suffix(ident);
    let parent = &parent_struct;

    let trait_impls = if data.variants.is_empty() {
        quote! {
            impl #impl_generics #crate_name::HasEvents for #parent #type_generics #where_clause {
            fn events() -> Vec<#ident #type_generics> {
                use strum::IntoEnumIterator;
                vec![]
            }

            fn event_defs() -> Vec<#crate_name::EventDef> {
                use strum::IntoEnumIterator;
                vec![]
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::EventTrait for #ident #type_generics #where_clause {
            type EntityName = #parent #type_generics;

            fn def(&self) -> #crate_name::EventDef {
                   #crate_name::EventDef::new(
                    "",
                    <#parent as #crate_name::NamedType>::table_name(),
                    "",
                    "",
                    None,
                    false,
                    false
                )
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::EventType for #ident #type_generics #where_clause {
            fn table_name() -> &'static str {
                <#parent as #crate_name::NamedType>::table_name()
            }

            fn event_name(&self) -> &str {
                ""
            }
        }

        #[automatically_derived]
        impl #impl_generics std::str::FromStr for #ident #type_generics #where_clause {
            type Err = #err_type;

            fn from_str(s: &str) -> Result<#ident, #err_type> {
                Err(<#err_type>::new(s.to_owned()))
            }
        }

        #[automatically_derived]
        impl #impl_generics core::convert::AsRef<str> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                ""
            }
        }

        impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", "")
            }
        }

        #[automatically_derived]
        impl #impl_generics ::core::fmt::Debug for #ident #type_generics #where_clause {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                write!(f, "{}", "")
            }
        }

        #[automatically_derived]
        impl #impl_generics ::core::marker::Copy for #ident #type_generics #where_clause {}
        #[automatically_derived]
        impl #impl_generics ::core::clone::Clone for #ident #type_generics #where_clause {
            #[inline]
            fn clone(&self) -> #ident #type_generics {
                *self
            }
        }

        #[automatically_derived]
        impl #impl_generics ::core::cmp::PartialEq for #ident #type_generics #where_clause {
            #[inline]
            fn eq(&self, other: &#ident #type_generics) -> bool {
                ::core::mem::discriminant(self) == ::core::mem::discriminant(other)
            }
        }
        #[automatically_derived]
        impl #impl_generics ::core::cmp::Eq for #ident #type_generics #where_clause {}
        }
    } else {
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

            let then = attrs.then.as_ref().ok_or_else(|| {
                syn::Error::new_spanned(variant, "Event must specify 'then' action")
            })?;
            let then_str = quote!(#then).to_string();

            let comment = match attrs.comment {
                Some(c) => quote!(Some(#c.to_string())),
                None => quote!(None),
            };
            let overwrite = attrs.overwrite;
            let if_not_exists = attrs.if_not_exists;

            let def = quote! {
                #ident::#variant_name => #crate_name::EventDef::new(
                    #event_name,
                    <#parent as #crate_name::NamedType>::table_name(),
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

        quote! {
            impl #impl_generics #crate_name::HasEvents for #parent #type_generics #where_clause {
                fn events() -> Vec<#ident #type_generics> {
                    use strum::IntoEnumIterator;
                    #ident::iter().collect::<Vec<_>>()
                }

                fn event_defs() -> Vec<#crate_name::EventDef> {
                    use strum::IntoEnumIterator;
                    #ident::iter().map(|e| e.def()).collect()
                }
            }

            #[automatically_derived]
            impl #impl_generics #crate_name::EventTrait for #ident #type_generics #where_clause {
                type EntityName = #parent #type_generics;

                fn def(&self) -> #crate_name::EventDef {
                    match self {
                        #(#defs,)*
                    }
                }
            }

            #[automatically_derived]
            impl #impl_generics #crate_name::EventType for #ident #type_generics #where_clause {
                fn table_name() -> &'static str {
                    <#parent as #crate_name::NamedType>::table_name()
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

        #[automatically_derived]
        impl #impl_generics ::core::fmt::Debug for #ident #type_generics #where_clause {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    #(#ident::#variants => write!(f, "{}", #names),)*
                }
            }
        }

            #[automatically_derived]
            impl #impl_generics ::core::marker::Copy for #ident #type_generics #where_clause {}
            #[automatically_derived]
            impl #impl_generics ::core::clone::Clone for #ident #type_generics #where_clause {
                #[inline]
                fn clone(&self) -> #ident #type_generics {
                    *self
                }
            }

            #[automatically_derived]
            impl #impl_generics ::core::cmp::PartialEq for #ident #type_generics #where_clause {
                #[inline]
                fn eq(&self, other: &#ident #type_generics) -> bool {
                    ::core::mem::discriminant(self) == ::core::mem::discriminant(other)
                }
            }
            #[automatically_derived]
            impl #impl_generics ::core::cmp::Eq for #ident #type_generics #where_clause {}

            inventory::submit! {
                #crate_name::EventRegistration {
                    builder: || -> Vec<#crate_name::EventDef> {
                        use strum::IntoEnumIterator;
                        #ident::iter().map(|e| e.def()).collect()
                    },
                    type_id: std::any::TypeId::of::<#parent #type_generics>(),
                }
            }
        }
    };
    Ok(quote! {

        #trait_impls
    })
}
