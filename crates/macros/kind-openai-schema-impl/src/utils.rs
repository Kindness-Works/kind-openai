use serde_json::{json, Value};
use syn::{Attribute, Ident, Type};

/// Extracts the description to provide to the JSON schema by scraping and reading triple-slash doc comments.
/// This works on top-level structs, top-level enums, and individual struct fields. AFAIK it's not possible to
/// place descriptions on enum variants according to JSON schema (nor that it would even be useful to do so),
/// but it's worth looking into more one day.
pub fn get_description(attrs: &[Attribute]) -> Option<String> {
    let docs = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                match &attr.meta {
                    syn::Meta::NameValue(meta_name_value) => match &meta_name_value.value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) => Some(lit_str.value().trim().to_owned()),
                        _ => None,
                    },
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    if docs.is_empty() {
        None
    } else {
        Some(docs)
    }
}

pub fn has_top_level_serde_attr(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| matches!(get_serde_meta_item(attr), Ok(Some(_))))
}

pub fn has_repr_attr(attrs: &[Attribute]) -> Result<bool, syn::Error> {
    let mut repr = None;
    for attr in attrs {
        if attr.path().is_ident("repr") {
            if let syn::Meta::List(meta) = &attr.meta {
                meta.parse_nested_meta(|meta| {
                    const RECOGNIZED: &[&str] = &[
                        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
                        "i128", "isize",
                    ];
                    if RECOGNIZED.iter().any(|int| meta.path.is_ident(int)) {
                        repr = Some(meta.path.get_ident().unwrap().clone());
                        return Ok(());
                    }
                    if meta.path.is_ident("align") || meta.path.is_ident("packed") {
                        if meta.input.peek(syn::token::Paren) {
                            let arg;
                            syn::parenthesized!(arg in meta.input);
                            let _ = arg.parse::<proc_macro2::TokenStream>()?;
                        }
                        return Ok(());
                    }
                    Err(meta.error("unsupported repr for serde_repr enum"))
                })?;
            }
        }
    }

    Ok(repr.is_some())
}

pub fn get_serde_rename(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .find_map(|attr| match get_serde_meta_item(attr) {
            Ok(Some(tokens)) => {
                let tokens = tokens.to_string();
                match tokens.find("rename =") {
                    Some(rename_start) => {
                        let quote_start = tokens[rename_start..].find('"')?;
                        let start = rename_start + quote_start + 1;
                        let end = tokens[start..].find('"')?;
                        Some(tokens[start..start + end].to_string())
                    }
                    None => None,
                }
            }
            _ => None,
        })
}

pub fn get_serde_skip(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| match get_serde_meta_item(attr) {
        Ok(Some(tokens)) => tokens.to_string().contains("skip"),
        _ => false,
    })
}

fn get_serde_meta_item(attr: &Attribute) -> syn::Result<Option<&proc_macro2::TokenStream>> {
    if attr.path().is_ident("serde") {
        match &attr.meta {
            syn::Meta::List(meta) => Ok(Some(&meta.tokens)),
            bad => Err(syn::Error::new_spanned(bad, "unrecognized attribute")),
        }
    } else {
        Ok(None)
    }
}

#[derive(Clone)]
pub enum Schema {
    Subordinate(Ident),
    Inlined(Value),
}

/// This is the core util that underlies most of this crate, effectively this takes in a Rust type
/// and produces a corresponding JSON schema type for it.
pub fn get_field_type(ty: &Type) -> Result<Schema, syn::Error> {
    match ty {
        Type::Path(type_path) => {
            let segment =
                type_path.path.segments.last().ok_or_else(|| {
                    syn::Error::new_spanned(type_path, "Expected type path segment")
                })?;
            let type_name = segment.ident.to_string();
            match type_name.as_str() {
                "String" => Ok(Schema::Inlined(json!({ "type": "string" }))),
                "i32" | "i64" | "u32" | "u64" | "isize" | "usize" => {
                    Ok(Schema::Inlined(json!({ "type": "integer" })))
                }
                "f32" | "f64" => Ok(Schema::Inlined(json!({ "type": "number" }))),
                "bool" => Ok(Schema::Inlined(json!({ "type": "boolean" }))),
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_schema = get_field_type(inner_type)?;
                            let items = match inner_schema {
                                Schema::Subordinate(_name) => todo!(),
                                Schema::Inlined(schema) => schema,
                            };
                            Ok(Schema::Inlined(json!({
                                "type": "array",
                                "items": items,
                            })))
                        } else {
                            Err(syn::Error::new_spanned(
                                args,
                                "Expected a type argument for Vec",
                            ))
                        }
                    } else {
                        Err(syn::Error::new_spanned(
                            segment,
                            "Expected angle bracketed arguments for Vec",
                        ))
                    }
                }
                "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_schema = get_field_type(inner_type)?;
                            let schema_with_null = merge_with_null(inner_schema);
                            Ok(Schema::Inlined(schema_with_null))
                        } else {
                            Err(syn::Error::new_spanned(
                                args,
                                "Expected a type argument for Option",
                            ))
                        }
                    } else {
                        Err(syn::Error::new_spanned(
                            segment,
                            "Expected angle bracketed arguments for Option",
                        ))
                    }
                }
                _hopefully_an_enum => Ok(Schema::Subordinate(segment.ident.clone())),
            }
        }
        _ => Err(syn::Error::new_spanned(ty, "Unsupported type")),
    }
}

fn merge_with_null(schema: Schema) -> Value {
    match schema {
        Schema::Inlined(ref schema @ Value::Object(ref map)) => {
            if let Some(Value::String(type_str)) = map.get("type") {
                let mut map = map.clone();
                map.insert(
                    "type".to_string(),
                    Value::Array(vec![
                        Value::String(type_str.clone()),
                        Value::String("null".to_string()),
                    ]),
                );
                Value::Object(map.clone())
            } else {
                json!({
                    "anyOf": [
                        schema,
                        { "type": "null" }
                    ]
                })
            }
        }
        Schema::Inlined(schema) => json!({
            "anyOf": [
                schema,
                { "type": "null" }
            ]
        }),
        Schema::Subordinate(_) => todo!(),
    }
}
