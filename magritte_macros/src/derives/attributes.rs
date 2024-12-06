use deluxe::{ExtractAttributes, ParseMetaItem};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::fmt::{Display, Formatter};
use heck::ToSnakeCase;
use serde::ser::Error;
use syn::{DeriveInput, Expr, ExprArray, LitStr, Path};

#[derive(Default, ParseMetaItem)]
pub struct AsSelect {
    #[deluxe(default)]
    pub select: Option<Expr>,
    #[deluxe(default)]
    pub from: Option<Expr>,
    #[deluxe(rename = where, default)]
    pub where_: Option<Expr>,
    #[deluxe(default)]
    pub group_by: Option<String>,
}

impl ToTokens for AsSelect {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let AsSelect {
            select,
            from,
            where_,
            group_by,
        } = self;
        let expanded = quote! {
            AsSelect {
                select: #select,
                from: #from,
                where_: #where_,
                group_by: #group_by,
            }
        };
        tokens.extend(expanded);
    }
}
impl Display for AsSelect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut query = String::new();
        if let Some(select) = &self.select {
            query.push_str(&format!("{}", quote!(#select)));
        } else {
            query.push_str("*");
        }
        if let Some(from) = &self.from {
            query.push_str(&format!(" FROM {}", quote!(#from)));
        }
        if let Some(where_clause) = &self.where_ {
            query.push_str(&format!(" WHERE {}", quote!(#where_clause)));
        }
        if let Some(group_by) = &self.group_by {
            query.push_str(&format!(" GROUP BY {}", quote!(#group_by)));
        }
        write!(f, "{}", query)
    }
}
#[derive(Default, ExtractAttributes)]
#[deluxe(attributes(table))]
pub struct Table {
    #[deluxe(default)]
    pub name: Option<String>,
    #[deluxe(default)]
    pub schema: Option<String>,
    #[deluxe(default)]
    pub drop: bool,
    #[deluxe(default)]
    pub overwrite: bool,
    #[deluxe(default)]
    pub if_not_exists: bool,
    #[deluxe(default)]
    pub permissions: Option<ExprArray>,
    #[deluxe(default)]
    pub changefeed: Option<String>,
    #[deluxe(default)]
    pub include_original: bool,
    #[deluxe(default)]
    pub comment: Option<String>,
    #[deluxe(default)]
    pub as_select: Option<AsSelect>,
}

impl ToTokens for Table {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Table {
            name,
            schema,
            drop,
            overwrite,
            if_not_exists,
            permissions,
            changefeed,
            include_original,
            comment,
            as_select,
        } = self;
        let expanded = quote! {
            Table {
                name: #name,
                schema: #schema,
                drop: #drop,
                overwrite: #overwrite,
                if_not_exists: #if_not_exists,
                permissions: #permissions,
                changefeed: #changefeed,
                include_original: #include_original,
                comment: #comment,
                as_select: #as_select,
            }
        };
        tokens.extend(expanded);
    }
}

#[derive(Default, ExtractAttributes)]
#[deluxe(attributes(column))]
pub struct Column {
    #[deluxe(default)]
    pub ignore: bool,
    #[deluxe(default)]
    pub name: Option<String>,
    #[deluxe(rename = type)]
    pub type_name: Option<String>,
    #[deluxe(default)]
    pub nullable: bool,
    #[deluxe(default)]
    pub flexible: bool,
    #[deluxe(default)]
    pub default: Option<Expr>,
    #[deluxe(default)]
    pub value: Option<Expr>,
    #[deluxe(default)]
    pub assert: Option<Expr>,
    #[deluxe(default)]
    pub permissions: Option<ExprArray>,
    #[deluxe(default)]
    pub readonly: bool,
    #[deluxe(default)]
    pub comment: Option<String>,
}

impl ToTokens for Column {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Column {
            ignore,
            name,
            type_name,
            nullable,
            flexible,
            default,
            value,
            assert,
            permissions,
            readonly,
            comment,
        } = self;
        let expanded = quote! {
            Column {
                ignore: #ignore,
                name: #name,
                type_name: #type_name,
                nullable: #nullable,
                flexible: #flexible,
                default: #default,
                value: #value,
                assert: #assert,
                permissions: #permissions,
                readonly: #readonly,
                comment: #comment,
            }
        };
        tokens.extend(expanded);
    }
}

#[derive(Default, ExtractAttributes)]
#[deluxe(attributes(event))]
pub struct Event {
    #[deluxe(default)]
    pub name: Option<String>,
    pub table: Option<String>,
    pub when: Option<LitStr>,
    pub then: Option<LitStr>,
    #[deluxe(default)]
    pub overwrite: bool,
    #[deluxe(default)]
    pub if_not_exists: bool,
    #[deluxe(default)]
    pub comment: Option<String>,
}

impl ToTokens for Event {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Event {
            name,
            table,
            when,
            then,
            overwrite,
            if_not_exists,
            comment,
        } = self;
        let expanded = quote! {
            Event {
                name: #name,
                table: #table,
                when: #when,
                then: #then,
                overwrite: #overwrite,
                if_not_exists: #if_not_exists,
                comment: #comment,
            }
        };
        tokens.extend(expanded);
    }
}

#[derive(Default, ParseMetaItem)]
pub struct BM25 {
    #[deluxe(default)]
    pub k1: Option<f64>,
    #[deluxe(default)]
    pub b: Option<f64>,
}

impl ToTokens for BM25 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let BM25 { k1, b } = self;
        let expanded = quote! {
            BM25 {
                k1: #k1,
                b: #b,
            }
        };
        tokens.extend(expanded);
    }
}

#[derive(Default, ParseMetaItem)]
pub struct Search {
    #[deluxe(default)]
    pub analyzer: Option<String>,
    #[deluxe(default)]
    pub bm25: Option<BM25>,
    #[deluxe(default)]
    pub highlights: Option<bool>,
}

impl ToTokens for Search {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Search { analyzer, bm25, highlights } = self;
        let expanded = quote! {
            Search {
                analyzer: #analyzer,
                bm25: #bm25,
                highlights: #highlights,
            }
        };
        tokens.extend(expanded);
    }
}

impl Display for Search {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bm25 = if let Some(bm25) = &self.bm25 {
            let mut bm25_ = "BM25".to_string();
            if let Some(k1) = bm25.k1 {
                if let Some(b) = bm25.b {
                    bm25_.push_str(format!("({}, {})", k1, b).as_str())
                }
            }
            bm25_
        } else {
            "".to_string()
        };
        let analyzer = if let Some(analyzer) = &self.analyzer {
            format!("ANALYZER {}", analyzer)
        } else {
            "".to_string()
        };
        let highlights = if let Some(highlights) = self.highlights {
            " HIGHLIGHTS"
        } else {
            ""
        };
        write!(f, "SEARCH {} {} {}", &analyzer, &bm25, highlights)
    }
}

#[derive(Default, ParseMetaItem)]
pub struct MTree {
    #[deluxe(default)]
    pub dimension: Option<u32>,
    #[deluxe(default)]
    pub vector_type: Option<String>,
    #[deluxe(default)]
    pub dist: Option<String>,
    #[deluxe(default)]
    pub capacity: Option<u32>,
}
impl ToTokens for MTree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let MTree {
            dimension,
            vector_type,
            dist,
            capacity,
        } = self;
        let expanded = quote! {
            MTree {
                dimension: #dimension,
                vector_type: #vector_type,
                dist: #dist,
                capacity: #capacity,
            }
        };
        tokens.extend(expanded);
    }
}
impl Display for MTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dimension = if let Some(dimension) = self.dimension {
            format!("DIMENSION {}", dimension)
        } else {
            return Err(core::fmt::Error::custom("Dimension must be provided"))
        };
        let vector_type = if let Some(vector_type) = &self.vector_type {
            format!(" TYPE {}", vector_type)
        } else {
            "".to_string()
        };
        let dist = if let Some(dist) = &self.dist {
            format!("DIST {}", dist)
        } else {
            "".to_string()
        };
        let capacity = if let Some(capacity) = self.capacity {
            format!(" CAPACITY {}", capacity)
        } else {
            "".to_string()
        };
        write!(
            f,
            "MTREE {}{}{}{}",
            dimension, vector_type, dist, capacity
        )
    }
}

#[derive(Default, ParseMetaItem)]
pub struct HNSW {
    #[deluxe(default)]
    pub dimension: Option<u32>,
    #[deluxe(default)]
    pub vector_type: Option<String>,
    #[deluxe(default)]
    pub dist: Option<String>,
    #[deluxe(default)]
    pub efc: Option<u32>,
    #[deluxe(default)]
    pub m: Option<u32>,
}
impl ToTokens for HNSW {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let HNSW {
            dimension,
            vector_type,
            dist,
            efc,
            m,
        } = self;
        let expanded = quote! {
            HNSW {
                dimension: #dimension,
                vector_type: #vector_type,
                dist: #dist,
                efc: #efc,
                m: #m,
            }
        };
        tokens.extend(expanded);
    }
}
impl Display for HNSW {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dimension = if let Some(dimension) = self.dimension {
            format!("DIMENSION {}", dimension)
        } else {
            return Err(core::fmt::Error::custom("Dimension must be provided"))
        };
        let vector_type = if let Some(vector_type) = &self.vector_type {
            format!(" TYPE {}", vector_type)
        } else {
            "".to_string()
        };
        let dist = if let Some(dist) = &self.dist {
            format!("DIST {}", dist)
        } else {
            "".to_string()
        };
        let efc = if let Some(efc) = self.efc {
            format!(" EFC {}", efc)
        } else {
            "".to_string()
        };
        let m = if let Some(m) = self.m {
            format!(" M {}", m)
        } else {
            "".to_string()
        };
        write!(
            f,
            "HNSW {}{}{}{}{}",
            dimension, vector_type, dist, efc, m
        )
    }
}
#[derive(Default, ExtractAttributes)]
#[deluxe(attributes(index))]
pub struct Index {
    #[deluxe(default)]
    pub name: Option<String>,
    #[deluxe(default)]
    pub table: Option<String>,
    #[deluxe(default)]
    pub overwrite: bool,
    #[deluxe(default)]
    pub if_not_exists: bool,
    #[deluxe(default)]
    pub fields: Option<ExprArray>,
    #[deluxe(default)]
    pub columns: Option<ExprArray>,
    #[deluxe(default)]
    pub unique: bool,
    #[deluxe(default)]
    pub use_table: bool,
    #[deluxe(default)]
    pub search: Option<Search>,
    #[deluxe(default)]
    pub mtree: Option<MTree>,
    #[deluxe(default)]
    pub hnsw: Option<HNSW>,
    #[deluxe(default)]
    pub concurrently: bool,
    #[deluxe(default)]
    pub comment: Option<String>,
}

impl ToTokens for Index {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Index {
            name,
            table,
            overwrite,
            if_not_exists,
            fields,
            columns,
            unique,
            use_table,
            search,
            mtree,
            hnsw,
            concurrently,
            comment,
        } = self;
        let expanded = quote! {
            Index {
                name: #name,
                table: #table,
                overwrite: #overwrite,
                if_not_exists: #if_not_exists,
                fields: #fields,
                columns: #columns,
                unique: #unique,
                use_table: #use_table,
                search: #search,
                mtree: #mtree,
                hnsw: #hnsw,
                concurrently: #concurrently,
                comment: #comment,
            }
        };
        tokens.extend(expanded);
    }
}

#[derive(Default, ExtractAttributes)]
#[deluxe(attributes(edge))]
pub struct Edge {
    #[deluxe(default)]
    pub name: Option<String>,
    #[deluxe(default)]
    pub from: Option<Path>,
    #[deluxe(default)]
    pub to: Option<Path>,
    #[deluxe(default)]
    pub enforced: bool,
    #[deluxe(default)]
    pub schema: Option<String>,
    #[deluxe(default)]
    pub permissions: Option<ExprArray>,
    #[deluxe(default)]
    pub overwrite: bool,
    #[deluxe(default)]
    pub if_not_exists: bool,
    #[deluxe(default)]
    pub drop: bool,
    #[deluxe(default)]
    pub changefeed: Option<String>,
    #[deluxe(default)]
    pub include_original: bool,
    #[deluxe(default)]
    pub comment: Option<String>,
    #[deluxe(default)]
    pub as_select: Option<AsSelect>,
}

impl ToTokens for Edge {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Edge {
            name,
            from,
            to,
            enforced,
            schema,
            permissions,
            overwrite,
            if_not_exists,
            drop,
            changefeed,
            include_original,
            comment,
            as_select,
        } = self;

        let expanded = quote! {
            Edge {
                name: #name,
                from: #from,
                to: #to,
                enforced: #enforced,
                schema: #schema,
                permissions: #permissions,
                overwrite: #overwrite,
                if_not_exists: #if_not_exists,
                drop: #drop,
                comment: #comment,
                changefeed: #changefeed,
                include_original: #include_original,
                as_select: #as_select
            }
        };
        tokens.extend(expanded);
    }
}

#[derive(Default, ExtractAttributes)]
#[deluxe(attributes(relate))]
pub struct Relate {
    #[deluxe(default)]
    pub from: Option<Path>,
    #[deluxe(default, rename = in)]
    pub in_id: Option<String>,
    #[deluxe(default)]
    pub to: Option<Path>,
    #[deluxe(default, rename = out)]
    pub out_id: Option<String>,
    #[deluxe(default)]
    pub edge: Option<Path>,
    #[deluxe(default)]
    pub content: Option<Expr>,
}

impl ToTokens for Relate {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Relate {
            from,
            in_id,
            to,
            out_id,
            edge,
            content,
        } = self;
        let expanded = quote! {
            Relate {
                from: #from
                in_id: #in_id,
                to: #to,
                out_id: #out_id,
                edge: #edge,
                content: #content,
            }
        };
        tokens.extend(expanded);
    }
}

// Helper function to extract generics from DeriveInput
pub(crate) fn split_generics(input: &DeriveInput) -> (TokenStream, TokenStream, TokenStream) {
    let DeriveInput { generics, .. } = input;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    (
        quote!(#impl_generics),
        quote!(#type_generics),
        quote!(#where_clause),
    )
}

pub trait HasTableName {
    fn table_name(&self) -> Option<String>;
}

pub fn resolve_table_name(attrs: &impl HasTableName, ident: &syn::Ident) -> String {
    attrs
        .table_name()
        .unwrap_or_else(|| ident.to_string().to_snake_case())
}

impl HasTableName for Table {
    fn table_name(&self) -> Option<String> {
        self.name.as_ref().map(|lit| lit.to_string())
    }
}

impl HasTableName for Edge {
    fn table_name(&self) -> Option<String> {
        self.name.as_ref().map(|lit| lit.to_string())
    }
}

pub fn resolve_parent_table_name(event_enum_ident: &syn::Ident) -> String {
    let name = event_enum_ident.to_string();
    if name.ends_with("Events") {
        name[..name.len() - 6].to_string()
    } else {
        name
    }
}