use deluxe::ExtractAttributes;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Data, Fields};
use crate::derives::{
    attributes::{Entity, Table, resolve_table_name, HasTableName},
    expand_derive_table, expand_derive_event, expand_derive_index, expand_derive_relation,
};

pub struct DeriveEntity {
    ident: syn::Ident,
    event_ident: syn::Ident,
    index_ident: syn::Ident,
    relation_ident: syn::Ident,
    table_name: String,
}

pub fn expand_derive_entity(input: DeriveInput) -> syn::Result<TokenStream> {
    let entity = DeriveEntity::new(input.clone())?;
    
    // Generate the empty enums for events, indexes, and relations
    let event_ident = &entity.event_ident;
    let index_ident = &entity.index_ident;
    let relation_ident = &entity.relation_ident;
    let ident = &entity.ident;
    
    // Generate Columns type name from the struct name
    let column_ident = format_ident!("{}Columns", ident);

    // Create enum DeriveInputs for each associated type
    let mut event_input = input.clone();
    event_input.ident = event_ident.clone();
    event_input.data = Data::Enum(syn::DataEnum {
        enum_token: syn::token::Enum::default(),
        brace_token: syn::token::Brace::default(),
        variants: syn::punctuated::Punctuated::new(),
    });

    let mut index_input = input.clone();
    index_input.ident = index_ident.clone();
    index_input.data = Data::Enum(syn::DataEnum {
        enum_token: syn::token::Enum::default(),
        brace_token: syn::token::Brace::default(),
        variants: syn::punctuated::Punctuated::new(),
    });

    let mut relation_input = input.clone();
    relation_input.ident = relation_ident.clone();
    relation_input.data = Data::Enum(syn::DataEnum {
        enum_token: syn::token::Enum::default(),
        brace_token: syn::token::Brace::default(),
        variants: syn::punctuated::Punctuated::new(),
    });

    // Derive implementations for each enum and the table
    let table_impl = expand_derive_table(input.clone())?;
    let event_impl = expand_derive_event(event_input, Some(input.clone()))?;
    let index_impl = expand_derive_index(index_input, Some(input.clone()))?;
    let relation_impl = expand_derive_relation(relation_input, Some(input.clone()))?;

    // Combine entity trait implementation with all other implementations
    let entity_impl = entity.impl_entity_trait(&column_ident);

    Ok(quote! {
        #entity_impl
        #table_impl
        #event_impl
        #index_impl
        #relation_impl
    })
}

impl DeriveEntity {
    fn new(mut input: DeriveInput) -> Result<Self, syn::Error> {
        let entity_attr = Entity::extract_attributes(&mut input.attrs)?;
        let table_attr = Table::extract_attributes(&mut input.attrs)?;
        let ident = input.ident.clone();
        
        // Use table attributes for name resolution
        let table_name = resolve_table_name(&table_attr, &ident);
        
        // Use entity attributes for type names, or default to struct name + suffix
        let event_ident = entity_attr.event.unwrap_or_else(|| format_ident!("{}Events", ident));
        let index_ident = entity_attr.index.unwrap_or_else(|| format_ident!("{}Indexes", ident));
        let relation_ident = entity_attr
            .relation
            .unwrap_or_else(|| format_ident!("{}Relations", ident));
        
        Ok(DeriveEntity {
            ident,
            event_ident,
            index_ident,
            relation_ident,
            table_name,
        })
    }

    fn impl_entity_trait(&self, column_ident: &syn::Ident) -> TokenStream {
        let Self {
            ident,
            event_ident,
            index_ident,
            relation_ident,
            ..
        } = self;

        quote!(
            #[automatically_derived]
            impl magritte::prelude::EntityTrait for #ident {
                type Table = Self;
                type Events = #event_ident;
                type Columns = #column_ident;
                type Indexes = #index_ident;
                type Relations = #relation_ident;
            }
        )
    }
}