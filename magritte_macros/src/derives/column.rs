use super::{
    attributes::{resolve_table_name, split_generics, Column, Table},
    expr_array_to_vec,
};
use crate::conversion::type_to_surrealdb_type;
use deluxe::ExtractAttributes;
use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Fields};

fn extract_field_type(field: &Field) -> String {
    type_to_surrealdb_type(&field.ty)
}

pub fn expand_derive_column(mut input: DeriveInput) -> syn::Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = split_generics(&input);

    // Get the actual table name that will be used
    let attrs = Table::extract_attributes(&mut input.attrs)?;
    let entity_name = &input.ident;
    let table_name = resolve_table_name(&attrs, entity_name);
    let table_name_str = &*table_name;
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

        let column_name = field_attrs
            .name
            .clone()
            .unwrap_or_else(|| field_name.to_string().to_snake_case());

        let field_type_ = &field.ty;
        // Use our new type inference
        let field_type = extract_field_type(field);
        let type_name = field_attrs.type_name.clone().unwrap_or_else(|| field_type);

        let permissions = field_attrs
            .permissions
            .as_ref()
            .map(|expr_array| expr_array_to_vec(expr_array))
            .unwrap_or_else(|| quote!(vec![]));

        // Determine nullability based on field type if not explicitly set
        let is_nullable = field_attrs.nullable;

        let default = field_attrs
            .default
            .as_ref()
            .map(|expr| quote!(#expr));
        let assert = field_attrs
            .assert
            .as_ref()
            .map(|expr| quote!(#expr));
        let comment = field_attrs
            .comment
            .as_ref()
            .map(|expr| quote!(#expr));
        let readonly = field_attrs.readonly;
        let flexible = field_attrs.flexible;

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

        let def = quote! {
            #column_enum_name::#variant_name => ColumnDef::new(
                #column_name,
                #table_name,
                #type_name,
                #default_value,
                #assert_value,
                #permissions,
                #is_nullable,
                #readonly,
                #flexible,
                #comment_value
            )
        };

        column_variants.push(quote!(#variant_name));
        column_defs.push(def);
        column_names.push(column_name);
        column_types.push(type_name);
    }

    let err_type = quote!(magritte::prelude::ColumnFromStrErr);

    Ok(quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter, serde::Serialize, serde::Deserialize)]
        pub enum #column_enum_name #type_generics #where_clause {
            #(#column_variants),*,
            #[doc(hidden)]
            __Phantom(::std::marker::PhantomData<#entity_name #type_generics>)
        }

        #[automatically_derived]
        impl #impl_generics ColumnTrait for #column_enum_name #type_generics #where_clause {
            type EntityName = #entity_name #type_generics;

            fn def(&self) -> ColumnDef {
                match self {
                    #(#column_defs,)*
                    #column_enum_name::__Phantom(_) => unreachable!()
                }
            }
        }

        impl #impl_generics ColumnType for #column_enum_name #type_generics #where_clause {
            fn column_name(&self) -> & str {
                match self {
                    #(#column_enum_name::#column_variants => #column_names,)*
                    #column_enum_name::__Phantom(_) => unreachable!()
                }
            }

            fn table_name() -> &'static str {
                #table_name_str
            }

            fn column_type(&self) -> & str {
                match self {
                    #(#column_enum_name::#column_variants => #column_types,)*
                    #column_enum_name::__Phantom(_) => unreachable!()
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
                    #column_enum_name::__Phantom(_) => unreachable!(),
                }
            }
        }
        #[automatically_derived]
        impl #impl_generics AsRef<str> for #column_enum_name #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                match self {
                    #(#column_enum_name::#column_variants => #column_names,)*
                    #column_enum_name::__Phantom(_) => unreachable!(),
                }
            }
        }
    })
}
