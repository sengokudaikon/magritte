use std::time::Duration;
use crate::derives::attributes::{resolve_table_name, split_generics, Table};
use crate::derives::column::expand_derive_column;
use crate::derives::expr_array_to_vec;
use deluxe::ExtractAttributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand_derive_table(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = split_generics(&input);
    let mut attrs = input.attrs.clone();
    let table_attr = Table::extract_attributes(&mut attrs)?;

    // Use shared table name resolution
    let table_name = resolve_table_name(&table_attr, ident);
    let table_name_lit = quote!(#table_name);
    let table_name_str = &*table_name;
    let schema_type = table_attr.schema.unwrap_or("SCHEMAFULL".to_string());
    let permissions = table_attr
        .permissions
        .as_ref()
        .map(|expr_array| expr_array_to_vec(expr_array))
        .unwrap_or_else(|| quote!(vec![]));

    let as_select = if let Some(as_select) = &table_attr.as_select {
        let query = as_select.to_string();
        quote!(Some(#query))
    } else {
        quote!(None)
    };

    let changefeed = if let Some(duration) = &table_attr.changefeed {
        let duration: u64 = duration.parse().unwrap();
        let include_original = table_attr.include_original;
        quote!(Some((#duration, #include_original)))
    } else {
        quote!(None)
    };

    let overwrite = table_attr.overwrite;
    let if_not_exists = table_attr.if_not_exists;
    let drop = table_attr.drop;
    let comment = if let Some(comment) = &table_attr.comment {
        quote!(Some(#comment.to_string()))
    } else {
        quote!(None)
    };

    let def = quote! {
        TableDef::new(
            #table_name,
            #schema_type,
            #overwrite,
            #if_not_exists,
            #permissions,
            #drop,
            #as_select,
            #changefeed,
            #comment,
        )
    };

    let err_type = quote!(magritte::TableFromStrErr);

    // Generate column enum and its implementations
    let column_impl = expand_derive_column(input.clone())?;

    Ok(quote! {
        // Generate the column enum first
        #column_impl

        #[automatically_derived]
        impl #impl_generics TableTrait for #ident #type_generics #where_clause {
            fn def(&self) -> TableDef {
                #def
            }
        }

        #[automatically_derived]
        impl #impl_generics NamedType for #ident #type_generics #where_clause {
            fn table_name() -> &'static str {
                #table_name_str
            }
        }

        #[automatically_derived]
        impl #impl_generics TableType for #ident #type_generics #where_clause {
            fn schema_type() -> SchemaType {
                #schema_type.into()
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<str> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                #table_name_str
            }
        }
        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", #table_name_str)
            }
        }
    })
}
