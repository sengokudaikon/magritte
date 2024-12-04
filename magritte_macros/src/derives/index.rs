use crate::derives::attributes::{split_generics, Index};
use deluxe::ExtractAttributes;
use magritte_query::vector_search::{VectorDistance, VectorType};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

fn strip_indexes_suffix(ident: &syn::Ident) -> String {
    let name = ident.to_string();
    if name.ends_with("Indexes") {
        name[..name.len() - 7].to_string()
    } else {
        name
    }
}

pub fn expand_derive_index(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = split_generics(&input);

    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Index can only be used on enums",
            ))
        }
    };

    let mut index_variants = Vec::new();
    let mut index_defs = Vec::new();
    let mut index_names = Vec::new();
    let mut table_name = None;

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let attrs = Index::extract_attributes(&mut variant.attrs.clone())?;

        // Get and validate table name from attributes
        let current_table = attrs.table.as_ref().ok_or_else(|| {
            syn::Error::new_spanned(variant, "Index must specify 'table' attribute")
        })?;

        // Ensure all variants reference the same table
        if let Some(ref prev_table) = table_name {
            if prev_table != current_table {
                return Err(syn::Error::new_spanned(
                    variant,
                    "All indexes in an enum must reference the same table",
                ));
            }
        } else {
            table_name = Some(current_table.clone());
        }

        let index_name = attrs
            .name
            .clone()
            .unwrap_or_else(|| variant_name.to_string());

        let fields = attrs
            .fields
            .as_ref()
            .map(|expr_array| {
                let field_tokens: Vec<_> = expr_array
                    .elems
                    .iter()
                    .map(|expr| quote!(#expr).to_string())
                    .collect();
                quote!(Some(vec![#(#field_tokens.to_string()),*]))
            })
            .unwrap_or_else(|| quote!(None));

        let columns = attrs
            .columns
            .as_ref()
            .map(|expr_array| {
                let column_tokens: Vec<_> = expr_array
                    .elems
                    .iter()
                    .map(|expr| quote!(#expr).to_string())
                    .collect();
                quote!(Some(vec![#(#column_tokens.to_string()),*]))
            })
            .unwrap_or_else(|| quote!(None));

        let specifics = if let Some(search) = &attrs.search {
            search.to_string()
        } else if let Some(mtree) = &attrs.mtree {
            mtree.to_string()
        } else if let Some(hnsw) = &attrs.hnsw {
            hnsw.to_string()
        } else {
            "".to_string()
        };
        let overwrite = attrs.overwrite;
        let use_table = attrs.use_table;
        let if_not_exists = attrs.if_not_exists;
        let unique = attrs.unique;
        let comment = match attrs.comment {
            Some(c) => quote!(Some(#c.to_string())),
            None => quote!(None),
        };
        let concurrently = attrs.concurrently;

        let def = quote! {
            #ident::#variant_name => {
                IndexDef::new(
                    #index_name.to_string(),
                    #current_table.to_string(),
                    #fields,
                    #columns,
                    #overwrite,
                    #use_table,
                    #if_not_exists,
                    #unique,
                    #specifics.to_string(),
                    #comment,
                    #concurrently,
                )
            }
        };
        index_variants.push(quote!(#variant_name));
        index_names.push(index_name);
        index_defs.push(def);
    }

    let table_name = table_name.expect("Table name must be specified");
    // Get parent struct name by stripping "Indexes" from enum name
    let parent_struct_name = strip_indexes_suffix(ident);
    let parent = format_ident!("{}", parent_struct_name);

    let err_type = quote!(magritte::IndexFromStrErr);
    let trait_impls = quote! {

        impl #impl_generics #parent #type_generics #where_clause {
            pub fn indexes() -> impl Iterator<Item = #ident #type_generics> {
                use strum::IntoEnumIterator;
                #ident::iter()
            }
        }

        #[automatically_derived]
        impl #impl_generics magritte::prelude::IndexTrait for #ident #type_generics #where_clause {
            type EntityName = #parent #type_generics;

            fn def(&self) -> IndexDef {
                match self {
                    #(#index_defs,)*
                }
            }
        }

        #[automatically_derived]

        impl #impl_generics magritte::prelude::IndexType for #ident #type_generics #where_clause {
            fn table_name() -> &'static str {
                #table_name
            }

            fn index_name(&self) -> &str {
                match self {
                    #(#ident::#index_variants => #index_names,)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics std::str::FromStr for #ident #type_generics #where_clause {
            type Err = #err_type;

            fn from_str(s: &str) -> Result<#ident, #err_type> {
                match s {
                    #(s if s == #index_names => Ok(#ident::#index_variants),)*
                    _ => Err(<#err_type>::new(s.to_owned())),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics core::convert::AsRef<str> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                match self {
                    #(#ident::#index_variants => #index_names,)*
                }
            }
        }

        impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#ident::#index_variants => write!(f, "{}", #index_names),)*
                }
            }
        }
    };

    Ok(quote! {
        #trait_impls
    })
}
