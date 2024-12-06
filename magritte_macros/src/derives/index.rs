use crate::derives::attributes::{split_generics, Index};
use deluxe::ExtractAttributes;
use magritte_query::vector_search::{VectorDistance, VectorType};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

fn strip_indexes_suffix(ident: &syn::Ident) -> syn::Path {
    let name = ident.to_string();
    let table_name = if name.ends_with("Indexes") {
        name[..name.len() - 7].to_string()
    } else {
        name
    };
    syn::parse_str::<syn::Path>(&table_name).unwrap_or_else(|_| syn::parse_quote!(#ident))
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
    let parent_struct = strip_indexes_suffix(ident);
    let parent = &parent_struct;

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let attrs = Index::extract_attributes(&mut variant.attrs.clone())?;

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
                    <#parent as magritte::prelude::NamedType>::table_name(),
                    #fields,
                    #columns,
                    #overwrite,
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

    let err_type = quote!(magritte::IndexFromStrErr);
    let trait_impls = quote! {

        impl #impl_generics magritte::prelude::HasIndexes for #parent #type_generics #where_clause {
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
                <#parent as magritte::prelude::NamedType>::table_name()
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

        #[automatically_derived]
        impl #impl_generics ::core::fmt::Debug for #ident #type_generics #where_clause {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    #(#ident::#index_variants => write!(f, "{}", #index_names),)*
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
