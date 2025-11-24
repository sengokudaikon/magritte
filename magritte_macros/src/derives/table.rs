use crate::derives::attributes::{resolve_table_name, split_generics, Table};
use crate::derives::column::expand_derive_column;
use crate::derives::expr_array_to_vec;
use deluxe::ExtractAttributes;
use macro_helpers::get_crate_name;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, FieldsNamed};

pub fn expand_derive_table(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = split_generics(&input);
    let mut attrs = input.attrs.clone();
    let table_attr = Table::extract_attributes(&mut attrs)?;
    let crate_name = get_crate_name(false);

    // Use shared table name resolution
    let table_name = resolve_table_name(&table_attr, ident)?;
    if table_name == "Dummy" {
        return Ok(quote!());
    }

    // Check for required enums
    let indexes_ident = syn::Ident::new(&format!("{}Indexes", ident), ident.span());
    let events_ident = syn::Ident::new(&format!("{}Events", ident), ident.span());

    // Try to find the enums in the current scope
    let assert_enums = quote::quote_spanned! {ident.span()=>
        const _: () = {
            #[allow(dead_code)]
            const ERROR_MSG: &'static str = concat!(
                "Table `", stringify!(#ident), "` requires associated enums:\n",
                "#[derive(Index, Serialize, Deserialize, strum::EnumIter)]\n",
                "pub enum ", stringify!(#ident), "Indexes {}\n\n",
                "#[derive(Event, Serialize, Deserialize, strum::EnumIter)]\n",
                "pub enum ", stringify!(#ident), "Events {}\n",
                "Even if you don't need indexes or events, these empty enums are required."
            );

            // This will fail at compile time if the enums don't exist or don't implement required traits
            trait AssertEnums {
                type Indexes: #crate_name::IndexType;
                type Events: #crate_name::EventType;
            }

            // If traits aren't implemented, show the error at compile time
            impl AssertEnums for #ident {
                type Indexes = #indexes_ident;
                type Events = #events_ident;
            }
        };
    };

    let table_name_lit = quote!(#table_name);
    let table_name_str = &*table_name;
    let schema_type = table_attr.schema.unwrap_or("SCHEMAFULL".to_string());
    let permissions = match table_attr.permissions.as_ref() {
        None => quote!(None), //None,
        Some(elems) => {
            let perms = expr_array_to_vec(elems);
            quote!(#perms)
        }
    };

    let as_select = if let Some(as_select) = &table_attr.as_select {
        let query = as_select.to_string();
        quote!(Some(#query.to_string()))
    } else {
        quote!(None)
    };

    let changefeed = if let Some(duration) = &table_attr.changefeed {
        let duration: u64 = duration.parse().map_err(|e| {
            syn::Error::new_spanned(ident, format!("Invalid changefeed duration value: {}", e))
        })?;
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
        #crate_name::TableDef::new(
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

    let err_type = quote!(#crate_name::TableFromStrErr);
    let data = match &input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Table can only be derived for structs",
            ));
        }
    };

    let fields = match &data.fields {
        Fields::Named(FieldsNamed { named, .. }) => named,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Table can only be derived for structs with named fields",
            ));
        }
    };

    let has_id_field = fields.iter().any(|field| {
        field
            .ident
            .as_ref()
            .map_or(false, |ident| ident.to_string().to_lowercase() == "id")
    });

    if !has_id_field {
        return Err(syn::Error::new_spanned(
            input,
            "Table must have an 'id' field. Please implement the HasId trait manually.",
        ));
    }
    // Generate column enum and its implementations
    let column_impl = expand_derive_column(input.clone())?;

    let registration = quote! {
        inventory::submit! {
            #crate_name::TableRegistration {
                builder: || -> anyhow::Result<#crate_name::TableSnapshot> {
                    #crate_name::table_snapshot::<#ident>()
                },
                type_id: std::any::TypeId::of::<#ident>(),
            }
        }
    };

    let expanded = quote! {
        #assert_enums

        // Generate the column enum first
        #column_impl

        #[automatically_derived]
        impl #impl_generics #crate_name::NamedType for #ident #type_generics #where_clause {
            fn table_name() -> &'static str {
                #table_name_str
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::RecordType for #ident #type_generics #where_clause {}

        #[automatically_derived]
        impl #impl_generics #crate_name::TableType for #ident #type_generics #where_clause {
            fn schema_type() -> #crate_name::SchemaType {
                #schema_type.into()
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::TableTrait for #ident #type_generics #where_clause {
            fn def() -> #crate_name::TableDef {
                #def
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::HasId for #ident #type_generics #where_clause {
            fn id(&self) -> #crate_name::SurrealId<Self> {
                self.id.clone()
            }
        }

        #[automatically_derived]
        impl #impl_generics core::convert::AsRef<str> for #ident #type_generics #where_clause {
            #[inline]
            fn as_ref(&self) -> &str {
                #table_name_str
            }
        }
        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", #table_name_str)
            }
        }
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Debug for #ident #type_generics #where_clause {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                write!(f, "{}", #table_name_str)
            }
        }

        #registration
    };

    Ok(expanded)
}
