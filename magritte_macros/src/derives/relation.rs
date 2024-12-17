use super::{attributes::{split_generics, Relate}};
use deluxe::ExtractAttributes;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{DeriveInput, Path};
use syn::spanned::Spanned;
use macro_helpers::get_crate_name;

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
    let crate_name = get_crate_name(false);
    let data = match &input.data {
        syn::Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Relation can only be derived for enums",
            ));
        }
    };

    let parent_struct = strip_relations_suffix(ident);
    let parent = &parent_struct;
    let mut variant_match_arms = Vec::new();
    let mut variant_impls = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let mut attrs = Relate::extract_attributes(&mut variant.attrs.clone())?;
        let to_table = attrs.to.take().ok_or_else(|| {
            syn::Error::new_spanned(variant, "Relation must specify target Table")
        })?;

        let edge_table = attrs.edge.take().ok_or_else(|| {
            syn::Error::new_spanned(variant, "Relation must specify edge Table")
        })?;
        let load_strategy = match attrs.eager {
            true => quote!(Some(#crate_name::LoadStrategy::Eager)),
            false => quote!(Some(#crate_name::LoadStrategy::Lazy)),
        };

        let content = attrs
            .content
            .take()
            .map(|c| quote!(Some(#c.to_string())))
            .unwrap_or_else(|| quote!(None));

        let from = quote!(<#parent as #crate_name::NamedType>::table_name());
        let to = quote!(<#to_table as #crate_name::NamedType>::table_name());
        let edge_str = quote!(<#edge_table as #crate_name::NamedType>::table_name());
        let relation_struct_name = syn::Ident::new(
            &format!("{}{}Relation", ident, variant_name),
            variant.span(),
        );
        let def_impl = quote! {
            impl #crate_name::RelationTrait for #relation_struct_name {
                type Source = #parent #type_generics;
                type Target = #to_table;
                type Edge = #edge_table;

                fn def() -> #crate_name::RelationDef {
                    #crate_name::RelationDef::new(#from, #to, #edge_str, #content, #load_strategy)
                }
            }

            #[automatically_derived]
            impl #crate_name::RelationType for #relation_struct_name {
                fn relation_via() -> String {
                    Self::def().relation_name().to_string()
                }

                fn relation_from() -> String {
                    Self::def().relation_from().to_string()
                }

                fn relation_to() -> String {
                    Self::def().relation_to().to_string()
                }
            }

            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            pub struct #relation_struct_name;
        };

        variant_impls.push(def_impl);
        variant_match_arms.push(quote! {
            #ident::#variant_name => #relation_struct_name::def()
        });
    }

    let err_type = quote!(#crate_name::RelationFromStrErr);
    let trait_impls = quote! {
        #(#variant_impls)*
        impl #impl_generics #crate_name::HasRelations for #parent #type_generics #where_clause {
            fn relations() -> Vec<#ident #type_generics> {
                use strum::IntoEnumIterator;
                #ident::iter().collect()
            }

            fn relation_defs() -> Vec<#crate_name::RelationDef> {
                use strum::IntoEnumIterator;
                #ident::iter().map(|r| r.relation_def()).collect()
            }
        }

        impl #impl_generics #crate_name::Relations for #ident #type_generics #where_clause {}

        impl #impl_generics #ident #type_generics #where_clause {
            pub fn relation_def(&self) -> #crate_name::RelationDef {
                match self {
                    #(#variant_match_arms),*
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
