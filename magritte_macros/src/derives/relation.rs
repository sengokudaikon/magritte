use super::attributes::{split_generics, Relate};
use deluxe::ExtractAttributes;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::DeriveInput;

fn strip_relations_suffix(ident: &syn::Ident) -> String {
    let name = ident.to_string();
    if name.ends_with("Relations") {
        name[..name.len() - 9].to_string()
    } else {
        name
    }
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

    // Get parent struct name by stripping "Relations" from enum name
    let parent_struct_name = strip_relations_suffix(ident);
    let parent = format_ident!("{}", parent_struct_name);

    let mut relation_variants = Vec::new();
    let mut relation_defs = Vec::new();
    let mut edge_tables = Vec::new();
    let mut relation_strings = Vec::new();
    let mut from_strings = Vec::new();
    let mut to_strings = Vec::new();
    let mut table_name = None;

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let mut attrs = Relate::extract_attributes(&mut variant.attrs.clone())?;

        // Get and validate table name from attributes
        let current_table = attrs.from.as_ref().ok_or_else(|| {
            syn::Error::new_spanned(variant, "Relation must specify 'table' attribute")
        })?;

        // Ensure all variants reference the same table
        if let Some(ref prev_table) = table_name {
            if prev_table != current_table {
                return Err(syn::Error::new_spanned(
                    variant,
                    "All relations in an enum must reference the same table",
                ));
            }
        } else {
            table_name = Some(current_table.clone());
        }

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

        let from = format!("{}:{}", current_table, in_id);
        let to = format!("{}:{}", to_table, out_id);
        let def = quote! {
            #ident::#variant_name => {
                RelationDef::new(#from, #to, #edge_table, #content)
            }
        };
        let relation_string = format!(
            "{}->{}->{}",
            from, edge_table, to
        );

        relation_variants.push(quote!(#variant_name));
        relation_defs.push(def);
        edge_tables.push(edge_table);
        relation_strings.push(relation_string);
        from_strings.push(from);
        to_strings.push(to);
    }

    let table_name = table_name.expect("Table name must be specified");

    let err_type = quote!(magritte::RelationFromStrErr);
    let enum_def = quote! {
        #[derive(Debug, Copy, Clone, strum::EnumIter, PartialEq, Eq)]
        pub enum #ident #type_generics #where_clause {
            #(#relation_variants,)*
            #[doc(hidden)]
            __Phantom(::std::marker::PhantomData<#parent #type_generics>)
        }
    };

    let trait_impls = quote! {
        #[automatically_derived]

        impl #impl_generics magritte::prelude::RelationTrait for #ident #type_generics #where_clause {
            type EntityName = #parent #type_generics;

            fn def(&self) -> RelationDef {
                match self {
                    #(#relation_defs,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }

        #[automatically_derived]

        impl #impl_generics magritte::prelude::RelationType for #ident #type_generics #where_clause {
            fn relation_via(&self) -> &str {
                match self {
                    #(#ident::#relation_variants => #edge_tables,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }

            fn relation_from(&self) -> &str {
                match self {
                    #(#ident::#relation_variants => #from_strings,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }

            fn relation_to(&self) -> &str {
                match self {
                    #(#ident::#relation_variants => #to_strings,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }

        impl #impl_generics std::str::FromStr for #ident #type_generics #where_clause {
            type Err = #err_type;

            fn from_str(s: &str) -> Result<#ident, #err_type> {
                match s {
                    #(s if s == #relation_strings => Ok(#ident::#relation_variants),)*
                    _ => Err(<#err_type>::new(s.to_owned())),
                }
            }
        }

        impl #impl_generics core::convert::AsRef<str> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                match self {
                    #(#ident::#relation_variants => #relation_strings,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }

        impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#ident::#relation_variants => write!(f, "{}", #relation_strings),)*
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
