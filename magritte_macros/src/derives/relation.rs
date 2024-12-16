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

    let mut variant_impls = Vec::new();
    let mut variant_refs = Vec::new();
    let mut variant_idents = Vec::new();

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
        let load_strategy = match attrs.eager {
            true => quote!(Some(#crate_name::LoadStrategy::Eager)),
            false => quote!(Some(#crate_name::LoadStrategy::Lazy)),
        };

        let content = attrs
            .content
            .take()
            .map(|c| quote!(Some(#c.to_string())))
            .unwrap_or_else(|| quote!(None));

        let from_str = quote!(<#parent as #crate_name::NamedType>::table_name());
        let to_str = quote!(<#to_table as #crate_name::NamedType>::table_name());
        let edge_str = quote!(<#edge_table as #crate_name::NamedType>::table_name());
        let from = quote!(format!("{}:{}", #from_str, #in_id));
        let to = quote!(format!("{}:{}", #to_str, #out_id));
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
        };

        variant_impls.push(def_impl);
        variant_refs.push(quote! {
            #ident::#variant_name => {
                // Create a static instance of the relation struct
                static REL: #relation_struct_name = #relation_struct_name;
                &REL
            }
        });
        variant_idents.push(variant_name);
        variant_impls.push(quote! {
            #[allow(non_camel_case_types)]
            pub struct #relation_struct_name;
        });
    }

    let err_type = quote!(#crate_name::RelationFromStrErr);
    let trait_impls = quote! {
        impl #impl_generics #crate_name::HasRelations for #parent #type_generics #where_clause {
            fn relations() -> Vec<#ident #type_generics> {
                use strum::IntoEnumIterator;
                #ident::iter().collect::<Vec<_>>()
            }

            fn relation_defs() -> Vec<#crate_name::RelationDef> {
                #ident::iter().map(|r| r.as_relation().def_owned()).collect()
            }
        }

        impl #impl_generics #ident #type_generics #where_clause {
            pub fn as_relation(&self) -> &'static (dyn #crate_name::RelationTrait<Source=#parent #type_generics> + 'static) {
                match self {
                    #(#variant_refs,)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::RelationType for #ident #type_generics #where_clause {
            fn relation_via(&self) -> &str {
                self.as_relation().def_owned().relation_name()
            }

            fn relation_from(&self) -> String {
                self.as_relation().def_owned().relation_from().to_string()
            }

            fn relation_to(&self) -> String {
                self.as_relation().def_owned().relation_to().to_string()
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
        #(#variant_impls)*
        #trait_impls
    })
}
