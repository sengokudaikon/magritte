use crate::derives::attributes::{resolve_table_name, split_generics, Index, Table};
use deluxe::ExtractAttributes;
use magritte_query::vector_search::{VectorDistance, VectorType};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

pub fn expand_derive_index(
    input: DeriveInput,
    parent_input: Option<DeriveInput>,
) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = split_generics(&input);
    let mut attrs = input.attrs.clone();
    let table_attr = Table::extract_attributes(&mut attrs)?;

    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Index can only be used on enums",
            ))
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

    let mut index_variants = Vec::new();
    let mut index_defs = Vec::new();
    let mut index_names = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let attrs = Index::extract_attributes(&mut variant.attrs.clone())?;

        let index_name = attrs
            .name
            .as_ref()
            .map(|name| name.clone())
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
                    #index_name
                    #table_name,
                    #fields,
                    #columns,
                    #overwrite,
                    #use_table,
                    #if_not_exists,
                    #unique,
                    #specifics,
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
    let enum_def = quote! {
        #[derive(Debug, Copy, Clone, strum::EnumIter)]
        #[derive(PartialEq, Eq)]
        pub enum #ident #type_generics #where_clause {
            #(#index_variants,)*
            #[doc(hidden)]
            __Phantom(::std::marker::PhantomData<#parent_ident #type_generics>)
        }
    };

    let trait_impls = quote! {
        #[automatically_derived]
        impl #impl_generics IndexTrait for #ident #type_generics #where_clause {
            type EntityName = #parent_ident #type_generics;

            fn def(&self) -> IndexDef {
                match self {
                    #(#index_defs,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }

        impl #impl_generics IndexType for #ident #type_generics #where_clause {
            fn table_name() -> &'static str {
                #table_name_str
            }

            fn index_name(&self) -> & str {
                match self {
                    #(#ident::#index_variants => &*#index_names,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }

        impl #impl_generics std::str::FromStr for #ident #type_generics #where_clause {
            type Err = #err_type;

            fn from_str(s: &str) -> Result<#ident, #err_type> {
                match s {
                    #(s if s == &*#index_names => Ok(#ident::#index_variants),)*
                    _ => Err(<#err_type>::new(s.to_owned())),
                }
            }
        }

        impl #impl_generics AsRef<str> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                match self {
                    #(#ident::#index_variants => &*#index_names,)*
                    #ident::__Phantom(_) => unreachable!()
                }
            }
        }

        impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#ident::#index_variants => write!(f, "{}", #index_names),)*
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
