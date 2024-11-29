use super::attributes::{resolve_table_name, split_generics, Relate, Table};
use deluxe::ExtractAttributes;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::DeriveInput;

enum Error {
    InputNotEnum,
    Syn(syn::Error),
}

struct DeriveRelation {
    table_name: String,
    parent_ident: syn::Ident,
    ident: syn::Ident,
    generics: (TokenStream, TokenStream, TokenStream),
    variants: syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
}

pub fn expand_derive_relation(input: syn::DeriveInput, parent_input: Option<DeriveInput>) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();
    let generics = split_generics(&input);
    match DeriveRelation::new(input,parent_input, generics) {
        Ok(model) => model.expand(),
        Err(Error::InputNotEnum) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveRelation on enums");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
impl DeriveRelation {
    fn new(
        mut input: syn::DeriveInput,
        parent_input: Option<DeriveInput>,
        generics: (TokenStream, TokenStream, TokenStream),
    ) -> Result<Self, Error> {
        let variants = match input.data {
            syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
            _ => return Err(Error::InputNotEnum),
        };

        let table_attr = Table::extract_attributes(&mut input.attrs).map_err(Error::Syn)?;
        let ident = input.ident;

        let parent_ident = if let Some(parent_input) = parent_input {
            parent_input.ident
        } else {
            format_ident!("relation")
        };

        // Get Table name from parent struct's table attribute
        let table_name = resolve_table_name(&table_attr, &parent_ident);

        Ok(DeriveRelation {
            table_name,
            parent_ident,
            ident,
            generics,
            variants,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_relation_trait = self.impl_relation_trait()?;

        Ok(expanded_impl_relation_trait)
    }
    fn impl_relation_trait(&self) -> syn::Result<TokenStream> {
        let ident = &self.ident;
        let table_name = &self.table_name;
        let table_ident = &self.parent_ident;
        let parent = quote!(#table_ident);
        let mut relation_variants = Vec::new();
        let mut relation_defs = Vec::new();
        let mut to_tables = Vec::new();
        let mut in_ids = Vec::new();
        let mut out_ids = Vec::new();
        let mut edge_tables = Vec::new();
        let mut relation_strings = Vec::new();
        let mut from_strings = Vec::new();
        let mut to_strings = Vec::new();

        let (impl_generics, type_generics, where_clause) = &self.generics;

        for variant in &self.variants {
            let variant_name = &variant.ident;
            let mut attrs = Relate::extract_attributes(&mut variant.attrs.clone())?;

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

            let from_table = quote!(#table_ident::table_name());
            let to_table_name = quote!(#to_table::table_name());
            let edge_table_name = quote!(#edge_table::table_name());

            let from_table_temp = from_table.to_string();
            let to_table_temp = to_table_name.to_string();
            let from = format!("{}:{}", from_table_temp, in_id);
            let to = format!("{}:{}", to_table_temp, out_id);
            let def = quote! {
                #ident::#variant_name => {
                    RelationDef::new(#from, #to, #edge_table_name, #content)
                }
            };
            let relation_string = format!(
                "{}:{}->{}->{}:{}",
                from_table_temp, in_id, edge_table_name, to_table_temp, out_id
            );

            relation_variants.push(quote!(#variant_name));
            relation_defs.push(def);
            to_tables.push(to_table);
            in_ids.push(in_id);
            out_ids.push(out_id);
            edge_tables.push(edge_table_name);
            relation_strings.push(relation_string);
            from_strings.push(from);
            to_strings.push(to);
        }

        let err_type = quote!(magritte::RelationFromStrErr);
        let enum_def = quote! {
            #[derive(Debug, Copy, Clone, strum::EnumIter)]
            #[derive(PartialEq, Eq)]
            pub enum #ident #type_generics #where_clause {
                #(#relation_variants,)*
                #[doc(hidden)]
                __Phantom(::std::marker::PhantomData<#parent #type_generics>)
            }
        };

        let trait_impls = quote! {
            #[automatically_derived]
            impl #impl_generics RelationTrait for #ident #type_generics #where_clause {
                type EntityName = #parent #type_generics;

                fn def(&self) -> RelationDef {
                    match self {
                        #(#relation_defs,)*
                        #ident::__Phantom(_) => unreachable!()
                    }
                }
            }

            impl #impl_generics RelationType for #ident #type_generics #where_clause {
                fn relation_via(&self) -> &str {
                    match self {
                        #(#ident::#relation_variants => {
                            &*#edge_tables
                        },)*
                        #ident::__Phantom(_) => unreachable!()
                    }
                }

                fn relation_from(&self) -> &str {
                    match self {
                        #(#ident::#relation_variants => &*#from_strings,)*
                        #ident::__Phantom(_) => unreachable!()
                    }
                }

                fn relation_to(&self) -> &str {
                    match self {
                        #(#ident::#relation_variants => &*#to_strings,)*
                        #ident::__Phantom(_) => unreachable!()
                    }
                }
            }

            impl #impl_generics std::str::FromStr for #ident #type_generics #where_clause {
                type Err = #err_type;

                fn from_str(s: &str) -> Result<#ident, #err_type> {
                    match s {
                        #(s if s == &*#relation_strings => Ok(#ident::#relation_variants),)*
                        _ => Err(<#err_type>::new(s.to_owned())),
                    }
                }
            }

            impl #impl_generics AsRef<str> for #ident #type_generics #where_clause {
                fn as_ref(&self) -> &str {
                    match self {
                        #(#ident::#relation_variants => &*#relation_strings,)*
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

            unsafe impl Send for #ident {}
            unsafe impl Sync for #ident {}
        };

        Ok(quote! {
            #enum_def

            #trait_impls
        })
    }
}
