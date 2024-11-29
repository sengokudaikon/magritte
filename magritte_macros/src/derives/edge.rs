use super::attributes::{split_generics, Edge};
use crate::derives::{conversion, expand_derive_column};
use deluxe::ExtractAttributes;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

pub fn expand_derive_edge(mut input: DeriveInput) -> syn::Result<TokenStream> {

    // Verify this is a struct
    match input.data {
        Data::Struct(_) => {},
        _ => return Err(syn::Error::new_spanned(
            input,
            "Edge can only be derived for structs",
        )),
    }

    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = split_generics(&input);
    let attrs = Edge::extract_attributes(&mut input.attrs)?;

    // Get the actual edge name that will be used - either from edge(name=) or struct name
    let edge_name = attrs
        .name
        .as_ref()
        .map(|name| name.clone())
        .unwrap_or_else(|| ident.to_string().to_lowercase());
    let edge_name_lit = quote!(#edge_name);
    let from_type = attrs
        .from
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(ident, "Edge must specify 'from' type"))?;

    let to_type = attrs
        .to
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(ident, "Edge must specify 'to' type"))?;

    let schema_type = attrs.schema.unwrap_or("SCHEMAFULL".to_string());

    let permissions = attrs
        .permissions
        .as_ref()
        .map(|expr_array| conversion::expr_array_to_vec(expr_array))
        .unwrap_or_else(|| quote!(vec![]));

    let overwrite = attrs.overwrite;
    let if_not_exists = attrs.if_not_exists;
    let drop = attrs.drop;
    let enforced = attrs.enforced;

    // Generate column enum and its implementations
    let column_impl = expand_derive_column(input.clone())?;

    Ok(quote! {
        #column_impl

        #[automatically_derived]
        impl #impl_generics EdgeTrait for #ident #type_generics #where_clause {
            type EntityFrom = #from_type;
            type EntityTo = #to_type;

            fn def(&self) -> EdgeDef {
                EdgeDef::new(
                    #edge_name_lit.to_string(),
                    #from_type::table_name().to_string(),
                    #to_type::table_name().to_string(),
                    #schema_type,
                    #permissions,
                    #overwrite,
                    #if_not_exists,
                    #drop,
                    #enforced
                )
            }
        }

        #[automatically_derived]
        impl #impl_generics NamedType for #ident #type_generics #where_clause {
            fn table_name() -> &'static str {
                #edge_name_lit
            }
        }

        #[automatically_derived]
        impl #impl_generics EdgeType for #ident #type_generics #where_clause {
            fn edge_from(&self) -> &str {
                #from_type::table_name()
            }

            fn edge_to(&self) -> &str {
                #to_type::table_name()
            }

            fn is_enforced(&self) -> bool {
                #enforced
            }
        }

        #[automatically_derived]
        impl #impl_generics AsRef<str> for #ident #type_generics #where_clause {
            fn as_ref(&self) -> &str {
                #edge_name_lit
            }
        }

        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", #edge_name_lit)
            }
        }
    })
}
