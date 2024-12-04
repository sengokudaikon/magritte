use super::attributes::{split_generics, Relate};
use deluxe::ExtractAttributes;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{DeriveInput, Path};

fn strip_relations_suffix(ident: &syn::Ident) -> Path {
    let name = ident.to_string();
    let table_name = if name.ends_with("Relations") {
        name[..name.len() - 9].to_string()
    } else {
        name
    };
    syn::parse_str::<syn::Path>(&table_name).unwrap_or_else(|_| syn::parse_quote!(#ident))
}

pub fn expand_derive_relation(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = split_generics(&input);
    
    let data = match &input.data {
        syn::Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Relation can only be derived for enums",
            ));
        }
    };

    let mut relation_variants = Vec::new();
    let mut relation_defs = Vec::new();
    let mut relation_from = Vec::new();
    let mut relation_to = Vec::new();
    let mut relation_via = Vec::new();
    let parent_struct = strip_relations_suffix(ident);
    let parent = &parent_struct;

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let mut attrs = Relate::extract_attributes(&mut variant.attrs.clone())?;
        let in_id = attrs
            .in_id
            .take()
            .ok_or_else(|| syn::Error::new_spanned(variant, "Relation must specify in_id"))?;
        let to_table = attrs.to.take().ok_or_else(|| {
            syn::Error::new_spanned(variant, "Relation must specify target Table")
        })?;

        let out_id = attrs
            .out_id
            .take()
            .ok_or_else(|| syn::Error::new_spanned(variant, "Relation must specify out_id"))?;
        let edge_table = attrs.edge.take().ok_or_else(|| {
            syn::Error::new_spanned(variant, "Relation must specify edge Table")
        })?;

        let content = attrs
            .content
            .take()
            .map(|c| quote!(Some(#c.to_string())))
            .unwrap_or_else(|| quote!(None));

        let from_str = quote!(<#parent as magritte::prelude::NamedType>::table_name());
        let to_str = quote!(<#to_table as magritte::prelude::NamedType>::table_name());
        let edge_str = quote!(<#edge_table as magritte::prelude::NamedType>::table_name());
        let from = quote!(format!("{}:{}", #from_str, #in_id));
        let to = quote!(format!("{}:{}", #to_str, #out_id));
        let def = quote! {
            #ident::#variant_name => {
                RelationDef::new(#from, #to, #edge_str, #content)
            }
        };

        relation_variants.push(quote!(#variant_name));
        relation_defs.push(def);
        relation_from.push(from);
        relation_via.push(edge_str);
        relation_to.push(to);
    }

    let err_type = quote!(magritte::RelationFromStrErr);
    let trait_impls = quote! {
        #[automatically_derived]
        impl #impl_generics magritte::prelude::RelationTrait for #ident #type_generics #where_clause {
            type EntityName = #parent #type_generics;

            fn def(&self) -> RelationDef {
                match self {
                    #(#relation_defs,)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics magritte::prelude::RelationType for #ident #type_generics #where_clause {
            fn relation_via(&self) -> &str {
                match self {
                    #(#ident::#relation_variants => #relation_via,)*
                }
            }

            fn relation_from(&self) -> String {
                match self {
                    #(#ident::#relation_variants => #relation_from,)*
                }
            }

            fn relation_to(&self) -> String {
                match self {
                    #(#ident::#relation_variants => #relation_to,)*
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
    };

    Ok(quote! {

        #trait_impls
    })
}
