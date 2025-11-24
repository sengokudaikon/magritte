use heck::ToSnakeCase;
use std::fmt::Display;
use syn::{DataEnum, Fields, Lit, Type, TypeArray, TypePath};
use magritte_core::{FieldType, Literal};

fn handle_set_type(args: &syn::PathArguments) -> FieldType {
    if let syn::PathArguments::AngleBracketed(args) = args {
        let mut types = args.args.iter();
        if let Some(syn::GenericArgument::Type(inner_type)) = types.next() {
            let base_type = type_to_surrealdb_type(inner_type);
            // Check for size parameter
            if let Some(syn::GenericArgument::Type(Type::Path(size_type))) = types.next() {
                if let Some(size_seg) = size_type.path.segments.last() {
                    if let Ok(size) = size_seg.ident.to_string().parse::<u64>() {
                        return FieldType::Set(Box::new(base_type), Some(size));
                    }
                }
            }
            return FieldType::Set(Box::new(base_type), None);
        }
    }
    FieldType::Set(Box::new(FieldType::Any), None)
}

fn handle_enum_type(data: &DataEnum) -> FieldType {
    let variants: Vec<FieldType> = data
        .variants
        .iter()
        .filter_map(|variant| {
            match &variant.fields {
                Fields::Unit => Some(FieldType::Literal(Literal::String(
                    variant.ident.to_string(),
                ))),
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
        return FieldType::Any;
    }
    FieldType::Either(variants)
}

pub fn type_to_surrealdb_type(ty: &Type) -> FieldType {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let segments: Vec<_> = path.segments.iter().collect();
            match segments.as_slice() {
                // Primitive types
                [seg] if seg.ident == "bool" => FieldType::Bool,
                [seg] if seg.ident == "i8" => FieldType::Int,
                [seg] if seg.ident == "i16" => FieldType::Int,
                [seg] if seg.ident == "i32" => FieldType::Int,
                [seg] if seg.ident == "i64" => FieldType::Int,
                [seg] if seg.ident == "u8" => FieldType::Int,
                [seg] if seg.ident == "u16" => FieldType::Int,
                [seg] if seg.ident == "u32" => FieldType::Int,
                [seg] if seg.ident == "u64" => FieldType::Int,
                [seg] if seg.ident == "f32" => FieldType::Float,
                [seg] if seg.ident == "f64" => FieldType::Float,
                [seg] if seg.ident == "String" => FieldType::String,
                [seg] if seg.ident == "str" => FieldType::String,
                [seg] if seg.ident == "char" => FieldType::String,

                [seg] if seg.ident == "Bytes" => FieldType::Bytes,
                [seg] if seg.ident == "Vec<u8>" => FieldType::Bytes,
                [seg] if seg.ident == "&[u8]" => FieldType::Bytes,

                // Optional types
                [seg] if seg.ident == "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            FieldType::Option(Box::new(type_to_surrealdb_type(inner_type)))
                        } else {
                            FieldType::Any
                        }
                    } else {
                        FieldType::Any
                    }
                }

                // Vec/Array types
                [seg] if seg.ident == "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            FieldType::Array(Box::new(type_to_surrealdb_type(inner_type)), None)
                        } else {
                            FieldType::Any
                        }
                    } else {
                        FieldType::Any
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
                                let table_name = seg.ident.to_string().to_snake_case();
                                FieldType::Record(table_name)
                            } else {
                                FieldType::Record("".to_string())
                            }
                        } else {
                            FieldType::Any
                        }
                    } else {
                        FieldType::Any
                    }
                }

                // Feature-gated types
                [ns, seg] if ns.ident == "chrono" && seg.ident == "DateTime" => FieldType::Datetime,
                [ns, seg] if ns.ident == "rust_decimal" && seg.ident == "Decimal" => {
                    FieldType::Decimal
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "Point" => {
                    FieldType::Geometry("point".to_string())
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "LineString" => {
                    FieldType::Geometry("linestring".to_string())
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "Polygon" => {
                    FieldType::Geometry("polygon".to_string())
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "MultiPoint" => {
                    FieldType::Geometry("multipoint".to_string())
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "MultiLineString" => {
                    FieldType::Geometry("multilinestring".to_string())
                }
                [ns, seg] if ns.ident == "geo" && seg.ident == "MultiPolygon" => {
                    FieldType::Geometry("multipolygon".to_string())
                }
                [ns, seg] if ns.ident == "uuid" && seg.ident == "Uuid" => FieldType::Uuid,
                [seg] if seg.ident == "Datetime" => FieldType::Datetime,
                [seg] if seg.ident == "Decimal" => FieldType::Decimal,
                [seg] if seg.ident == "Point" => FieldType::Point,
                [seg] if seg.ident == "Uuid" => FieldType::Uuid,

                // Namespaced types
                [ns, seg] if ns.ident == "std" && seg.ident == "Duration" => FieldType::Duration,
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Uuid" => FieldType::Uuid,
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Bytes" => FieldType::Bytes,
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Number" => FieldType::Number,
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Object" => FieldType::Object,
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Array" => {
                    FieldType::Array(Box::new(FieldType::Any), None)
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Geometry" => {
                    FieldType::Geometry("feature".to_string())
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Duration" => {
                    FieldType::Duration
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Operation" => {
                    FieldType::Object
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Strand" => FieldType::String,
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "Datetime" => {
                    FieldType::Datetime
                }
                [ns, module, seg]
                    if ns.ident == "surrealdb" && module.ident == "sql" && seg.ident == "Thing" =>
                {
                    FieldType::Record("".to_string())
                }
                [ns, module, seg]
                    if ns.ident == "surrealdb"
                        && module.ident == "sql"
                        && seg.ident == "Datetime" =>
                {
                    FieldType::Datetime
                }
                [ns, module, seg]
                    if ns.ident == "surrealdb" && module.ident == "sql" && seg.ident == "Uuid" =>
                {
                    FieldType::Uuid
                }
                [ns, seg] if ns.ident == "surrealdb" && seg.ident == "RecordId" => {
                    FieldType::Record("".to_string())
                }

                [ns, seg]
                    if ns.ident == "std" && (seg.ident == "HashSet" || seg.ident == "BTreeSet") =>
                {
                    handle_set_type(&seg.arguments)
                }

                // Default to any for unknown types
                _ => FieldType::Any,
            }
        }
        Type::Array(TypeArray { elem, len, .. }) => {
            let len_value = match len {
                syn::Expr::Lit(lit) => {
                    if let Lit::Int(int) = &lit.lit {
                        Some(int.base10_digits().parse::<u64>().unwrap_or(0))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            FieldType::Array(Box::new(type_to_surrealdb_type(elem)), len_value)
        }
        _ => FieldType::Any,
    }
}
