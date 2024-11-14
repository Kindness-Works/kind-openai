use serde_json::{json, Value};
use syn::{Attribute, Type};

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

/// Friendlier JSON-schema based representation of a Rust type.
/// There is an oddity with OpenAI's JSON schema where each variant MUST have `required`,
/// while at the same time also requiring that `required` is also true.
///
/// Including the `required` field here is mostly for clarity, but we could just hardcore it to true
/// elsewhere if we wanted.
///
/// Representing `Option<T>` is done by creating a union of `T` and a JSON `null`
pub struct FieldSchema {
    pub schema: Value,
    pub required: bool,
}

/// This is the core util that underlies most of this crate, effectively this takes in a Rust type
/// and produces a corresponding JSON schema type for it.
pub fn get_field_type(ty: &Type) -> Result<FieldSchema, syn::Error> {
    match ty {
        Type::Path(type_path) => {
            let segment =
                type_path.path.segments.last().ok_or_else(|| {
                    syn::Error::new_spanned(type_path, "Expected type path segment")
                })?;
            let type_name = segment.ident.to_string();
            match type_name.as_str() {
                "String" => Ok(FieldSchema {
                    schema: json!({ "type": "string" }),
                    required: true,
                }),
                "i32" | "i64" | "u32" | "u64" | "isize" | "usize" => Ok(FieldSchema {
                    schema: json!({ "type": "integer" }),
                    required: true,
                }),
                "f32" | "f64" => Ok(FieldSchema {
                    schema: json!({ "type": "number" }),
                    required: true,
                }),
                "bool" => Ok(FieldSchema {
                    schema: json!({ "type": "boolean" }),
                    required: true,
                }),
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_schema = get_field_type(inner_type)?;
                            Ok(FieldSchema {
                                schema: json!({
                                    "type": "array",
                                    "items": inner_schema.schema
                                }),
                                required: true,
                            })
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
                            let schema_with_null = merge_with_null(inner_schema.schema);
                            Ok(FieldSchema {
                                schema: schema_with_null,
                                required: false,
                            })
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
                _ => Err(syn::Error::new_spanned(
                    ty,
                    "Nested structs or enums are not supported",
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(ty, "Unsupported type")),
    }
}

fn merge_with_null(schema: Value) -> Value {
    match schema.clone() {
        Value::Object(mut map) => {
            if let Some(Value::String(type_str)) = map.get("type").cloned() {
                map.insert(
                    "type".to_string(),
                    Value::Array(vec![
                        Value::String(type_str),
                        Value::String("null".to_string()),
                    ]),
                );
                Value::Object(map)
            } else {
                json!({
                    "anyOf": [
                        schema,
                        { "type": "null" }
                    ]
                })
            }
        }
        _ => json!({
            "anyOf": [
                schema,
                { "type": "null" }
            ]
        }),
    }
}
