use proc_macro::Ident;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{DataEnum, Fields, Lit, Type, TypeArray, TypePath};
use magritte_query::types::TableType;

fn handle_set_type(args: &syn::PathArguments) -> String {
    if let syn::PathArguments::AngleBracketed(args) = args {
        let mut types = args.args.iter();
        if let Some(syn::GenericArgument::Type(inner_type)) = types.next() {
            let base_type = type_to_surrealdb_type(inner_type);
            // Check for size parameter
            if let Some(syn::GenericArgument::Type(Type::Path(size_type))) = types.next() {
                if let Some(size_seg) = size_type.path.segments.last() {
                    return format!("set<{}, {}>", base_type, size_seg.ident);
                }
            }
            return format!("set<{}>", base_type);
        }
    }
    "set".to_string()
}

fn handle_enum_type(data: &DataEnum) -> String {
    let variants: Vec<String> = data
        .variants
        .iter()
        .filter_map(|variant| {
            match &variant.fields {
                Fields::Unit => Some(format!("\"{}\"", variant.ident)),
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    let field = fields.unnamed.first()?;
                    Some(type_to_surrealdb_type(&field.ty))
                }
                Fields::Named(fields) if fields.named.len() == 1 => {
                    let field = fields.named.first()?;
                    Some(type_to_surrealdb_type(&field.ty))
                }
                _ => None, // Skip complex enum variants
            }
        })
        .collect();
    if variants.is_empty() {
        return "any".to_string();
    }
    variants.join(" | ")
}

pub fn type_to_surrealdb_type(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let segments: Vec<_> = path.segments.iter().collect();
            match segments.as_slice() {
                // Primitive types
                [seg] if seg.ident == "bool" => "bool".to_string(),
                [seg] if seg.ident == "i8" => "int".to_string(),
                [seg] if seg.ident == "i16" => "int".to_string(),
                [seg] if seg.ident == "i32" => "int".to_string(),
                [seg] if seg.ident == "i64" => "int".to_string(),
                [seg] if seg.ident == "u8" => "int".to_string(),
                [seg] if seg.ident == "u16" => "int".to_string(),
                [seg] if seg.ident == "u32" => "int".to_string(),
                [seg] if seg.ident == "u64" => "int".to_string(),
                [seg] if seg.ident == "f32" => "float".to_string(),
                [seg] if seg.ident == "f64" => "float".to_string(),
                [seg] if seg.ident == "String" => "string".to_string(),
                [seg] if seg.ident == "str" => "string".to_string(),
                [seg] if seg.ident == "char" => "string".to_string(),

                [seg] if seg.ident == "Bytes" => "bytes".to_string(),
                [seg] if seg.ident == "Vec<u8>" => "bytes".to_string(),
                [seg] if seg.ident == "&[u8]" => "bytes".to_string(),

                // Optional types
                [seg] if seg.ident == "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            format!("option<{}>", type_to_surrealdb_type(inner_type))
                        } else {
                            "any".to_string()
                        }
                    } else {
                        "any".to_string()
                    }
                }

                // Vec/Array types
                [seg] if seg.ident == "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            format!("array<{}>", type_to_surrealdb_type(inner_type))
                        } else {
                            "any".to_string()
                        }
                    } else {
                        "any".to_string()
                    }
                }

                // Set types
                [seg] if seg.ident == "Set" || seg.ident == "HashSet" => {
                    handle_set_type(&seg.arguments)
                }
                
                [seg] if seg.ident == "RecordRef" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(Type::Path(type_path))) =
                            args.args.first()
                        {
                            if let Some(seg) = type_path.path.segments.last() {
                                let table_name = seg.ident.to_string().to_lowercase();
                                format!("record<{}>", table_name)
                            } else {
                                "record<any>".to_string()
                            }
                        } else {
                            "any".to_string()
                        }
                    } else {
                        "any".to_string()
                    }
                }

                // Feature-gated types
                [ns, seg] if ns.ident == "chrono" && seg.ident == "DateTime" => {
                    "datetime".to_string()
                }
                [ns, seg] if ns.ident == "rust_decimal" && seg.ident == "Decimal" => {
                    "decimal".to_string()
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "Point" => {
                    "geometry<point>".to_string()
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "LineString" => {
                    "geometry<linestring>".to_string()
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "Polygon" => {
                    "geometry<polygon>".to_string()
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "MultiPoint" => {
                    "geometry<multipoint>".to_string()
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "MultiLineString" => {
                    "geometry<multilinestring>".to_string()
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "MultiPolygon" => {
                    "geometry<multipolygon>".to_string()
                }
                [ns, seg] if ns.ident == "uuid" && seg.ident == "Uuid" => "uuid".to_string(),
                [seg] if seg.ident == "Datetime" => "datetime".to_string(),
                [seg] if seg.ident == "Decimal" => "decimal".to_string(),
                [seg] if seg.ident == "Point" => "geometry<point>".to_string(),
                [seg] if seg.ident == "Uuid" => "uuid".to_string(),

                // Namespaced types
                [ns, seg] if ns.ident == "std" && seg.ident == "Duration" => "duration".to_string(),
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Uuid" => "uuid".to_string(),
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Bytes" => "bytes".to_string(),
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Number" => {
                    "number".to_string()
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Object" => {
                    "object".to_string()
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Array" => {
                    "array<any>".to_string()
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Geometry" => {
                    "geometry<feature>".to_string()
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Duration" => {
                    "duration".to_string()
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Operation" => {
                    "object".to_string()
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Strand" => {
                    "string".to_string()
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Datetime" => {
                    "datetime".to_string()
                }
                [ns, module, seg]
                    if ns.ident == "surrealdb" && module.ident == "sql" && seg.ident == "Thing" =>
                {
                    "record<any>".to_string()
                }
                [ns, module, seg]
                    if ns.ident == "surrealdb"
                        && module.ident == "sql"
                        && seg.ident == "Datetime" =>
                {
                    "datetime".to_string()
                }
                [ns, module, seg]
                    if ns.ident == "surrealdb" && module.ident == "sql" && seg.ident == "Uuid" =>
                {
                    "uuid".to_string()
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "RecordId" => {
                    "record<any>".to_string()
                }

                [ns, seg]
                    if ns.ident == "std" && (seg.ident == "HashSet" || seg.ident == "BTreeSet") =>
                {
                    handle_set_type(&seg.arguments)
                }

                // Default to any for unknown types
                _ => "any".to_string(),
            }
        }
        Type::Array(TypeArray { elem, len, .. }) => {
            let len_str = match len {
                syn::Expr::Lit(lit) => {
                    if let Lit::Int(int) = &lit.lit {
                        int.base10_digits()
                    } else {
                        "0"
                    }
                }
                _ => "0",
            };
            format!("array<{}, {}>", type_to_surrealdb_type(elem), len_str)
        }
        _ => "any".to_string(),
    }
}
