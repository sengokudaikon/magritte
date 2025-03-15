use super::{
    attributes::{resolve_table_name, split_generics, Column, Table},
    expr_array_to_vec,
};
use crate::conversion::type_to_surrealdb_type;
use deluxe::ExtractAttributes;
use heck::{ToPascalCase, ToSnakeCase};
use macro_helpers::get_crate_name;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_json::map;
use std::fmt::format;
use syn::{parse_quote, Data, DeriveInput, ExprArray, Field, Fields};

fn extract_field_type(field: &Field) -> String {
    type_to_surrealdb_type(&field.ty).to_string()
}

pub fn expand_derive_column(mut input: DeriveInput) -> syn::Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = split_generics(&input);

    // Get the actual table name that will be used
    let attrs = Table::extract_attributes(&mut input.attrs)?;
    let entity_name = &input.ident;

    let table_name = resolve_table_name(&attrs, entity_name)?;
    let table_name_str = &table_name;
    let column_enum_name = format_ident!("{}Columns", entity_name);
    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Column can only be derived for structs",
            ));
        }
    };

    let fields = match &data.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Column can only be derived for structs with named fields",
            ));
        }
    };

    let mut column_variants = Vec::new();
    let mut column_defs = Vec::new();
    let mut column_names = Vec::new();
    let mut column_types = Vec::new();
    let crate_name = get_crate_name(false);
    let entity_type: syn::Type = parse_quote!(#entity_name);
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let variant_name = format_ident!("{}", field_name.to_string().to_pascal_case());
        let variant_str = variant_name.to_string();

        // Try to extract column attributes if they exist, otherwise use defaults
        let field_attrs = if field.attrs.is_empty() {
            Column::default()
        } else {
            let mut field_attrs_clone = field.attrs.clone();
            Column::extract_attributes(&mut field_attrs_clone)?
        };
        if field_attrs.ignore {
            continue;
        }
        let column_name = field_attrs
            .name
            .clone()
            .unwrap_or_else(|| field_name.to_string().to_snake_case());
        let field_type = if field_name.to_string().to_lowercase() == "id" {
            format!("record<{}>", table_name_str)
        } else {
            extract_field_type(field)
        };
        let type_name = field_attrs.type_name.clone().unwrap_or(field_type);
        let permissions = match field_attrs.permissions.as_ref() {
            None => quote!(None), //None,
            Some(elems) => {
                let perms = expr_array_to_vec(elems);
                quote!(#perms)
            }
        };

        // Determine nullability based on field type if not explicitly set
        let is_nullable = field_attrs.nullable;

        let default = field_attrs.default.as_ref().map(|expr| quote!(#expr));
        let assert = field_attrs.assert.as_ref().map(|expr| quote!(#expr));
        let comment = field_attrs.comment.as_ref().map(|expr| quote!(#expr));
        let value = field_attrs.value.as_ref().map(|expr| quote!(#expr));
        let readonly = field_attrs.readonly;
        let flexible = field_attrs.flexible;
        let overwrite = field_attrs.overwrite;
        let if_not_exists = field_attrs.if_not_exists;
        let default_value = match default {
            Some(d) => quote!(Some(#d.to_string())),
            None => quote!(None),
        };

        let assert_value = match assert {
            Some(a) => quote!(Some(#a.to_string())),
            None => quote!(None),
        };

        let comment_value = match comment {
            Some(c) => quote!(Some(#c.to_string())),
            None => quote!(None),
        };

        let value_value = match value {
            Some(v) => quote!(Some(#v.to_string())),
            None => quote!(None),
        };

        let def = quote! {
            #column_enum_name::#variant_name => #crate_name::ColumnDef::new(
                #column_name,
                #table_name,
                #type_name,
                #default_value,
                #assert_value,
                #permissions,
                #value_value,
                #is_nullable,
                #readonly,
                #flexible,
                #overwrite,
                #if_not_exists,
                #comment_value
            )
        };
        column_variants.push(quote!(#variant_name));
        column_defs.push(def);
        column_names.push(column_name);
        column_types.push(type_name);
    }

    let err_type = quote!(#crate_name::ColumnFromStrErr);

    Ok(quote! {
        impl #impl_generics #crate_name::HasColumns for #entity_type #type_generics #where_clause {
            fn columns() -> Vec<#column_enum_name #type_generics> {
                use strum::IntoEnumIterator;
                #column_enum_name::iter().collect::<Vec<_>>()
            }

            fn column_defs() -> Vec<#crate_name::ColumnDef> {
                use strum::IntoEnumIterator;
                use #crate_name::ColumnTrait;
                #column_enum_name::iter().map(|r| r.def()).collect()
            }
        }

        #[automatically_derived]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, serde::Serialize, serde::Deserialize)]
        pub enum #column_enum_name #type_generics #where_clause {
            #(#column_variants),*,
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::ColumnTrait for #column_enum_name #type_generics #where_clause {
            type EntityName = #entity_type;

            fn def(&self) -> #crate_name::ColumnDef {
                match self {
                    #(#column_defs,)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::ColumnTypeLite for #column_enum_name #type_generics #where_clause {
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::ColumnType for #column_enum_name #type_generics #where_clause {
            fn column_name(&self) -> & str {
                match self {
                    #(#column_enum_name::#column_variants => #column_names,)*
                }
            }

            fn table_name() -> &'static str {
                #table_name_str

            }

            fn column_type(&self) -> & str {
                match self {
                    #(#column_enum_name::#column_variants => #column_types,)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics std::str::FromStr for #column_enum_name #type_generics #where_clause {
            type Err = #err_type;

            fn from_str(s: &str) -> Result<#column_enum_name, #err_type> {
                match s {
                    #(s if s == #column_names => Ok(#column_enum_name::#column_variants),)*
                    _ => Err(<#err_type>::new(s.to_owned())),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #column_enum_name #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#column_enum_name::#column_variants => write!(f, "{}", #column_names),)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics core::convert::AsRef<str> for #column_enum_name #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                match self {
                    #(#column_enum_name::#column_variants => #column_names,)*
                }
            }
        }
    })
}
