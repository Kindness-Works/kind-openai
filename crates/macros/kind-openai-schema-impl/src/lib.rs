use proc_macro::TokenStream;
use quote::quote;
use serde_json::{json, Value};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, GenericArgument, PathArguments,
    Type,
};

struct FieldSchema {
    schema: Value,
    required: bool,
}

#[proc_macro_derive(OpenAISchema)]
pub fn openai_schema_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let description = get_description(&input.attrs);

    let schema = match input.data {
        Data::Struct(data) => {
            let field_infos = match data.fields {
                Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|f| {
                        let field_name = f.ident.as_ref().unwrap().to_string();
                        let field_schema = get_field_type(&f.ty);
                        let description = get_description(&f.attrs);
                        FieldInfo {
                            name: field_name,
                            schema: field_schema.schema,
                            required: field_schema.required,
                            description,
                        }
                    })
                    .collect::<Vec<FieldInfo>>(),
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let field_name = i.to_string();
                        let field_schema = get_field_type(&f.ty);
                        let description = get_description(&f.attrs);
                        FieldInfo {
                            name: field_name,
                            schema: field_schema.schema,
                            required: field_schema.required,
                            description,
                        }
                    })
                    .collect::<Vec<FieldInfo>>(),
                Fields::Unit => Vec::new(),
            };

            let mut properties = serde_json::Map::new();
            for field in &field_infos {
                let mut field_schema = field.schema.clone();
                if let Some(description) = &field.description {
                    if let Some(obj) = field_schema.as_object_mut() {
                        obj.insert(
                            "description".to_string(),
                            Value::String(description.clone()),
                        );
                    }
                }
                properties.insert(field.name.clone(), field_schema);
            }

            let required: Vec<String> = field_infos
                .iter()
                .filter(|f| f.required)
                .map(|f| f.name.clone())
                .collect();

            json!({
                "name": name.to_string(),
                "description": description,
                "strict": true,
                "schema": {
                    "type": "object",
                    "properties": properties,
                    "required": required,
                    "additionalProperties": false
                }
            })
        }
        Data::Enum(data) => {
            let mut variant_values = Vec::new();
            let mut is_numeric_enum = true;
            for variant in data.variants.iter() {
                match &variant.fields {
                    syn::Fields::Unit => {
                        if let Some((_, expr)) = &variant.discriminant {
                            // Try to parse expr to get integer value
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Int(lit_int),
                                ..
                            }) = expr
                            {
                                let int_value = lit_int.base10_parse::<i64>().unwrap();
                                variant_values.push(json!(int_value));
                            } else {
                                // Discriminant is not integer literal
                                is_numeric_enum = false;
                                break;
                            }
                        } else {
                            // No discriminant
                            is_numeric_enum = false;
                            break;
                        }
                    }
                    _ => panic!("Only enums with unit variants are supported"),
                }
            }

            if is_numeric_enum && variant_values.len() == data.variants.len() {
                json!({
                    "name": name.to_string(),
                    "description": description,
                    "schema": {
                        "type": "number",
                        "enum": variant_values
                    }
                })
            } else {
                let variant_names = data
                    .variants
                    .iter()
                    .map(|variant| match &variant.fields {
                        syn::Fields::Unit => variant.ident.to_string(),
                        _ => panic!("Only enums with unit variants are supported"),
                    })
                    .collect::<Vec<String>>();

                json!({
                    "name": name.to_string(),
                    "description": description,
                    "schema": {
                        "type": "string",
                        "enum": variant_names
                    }
                })
            }
        }
        _ => panic!("Only structs and enums with unit variants are supported"),
    };

    let schema_str = serde_json::to_string(&schema).unwrap();

    let expanded = quote! {
        impl kind_openai::OpenAISchema for #name {
            fn openai_schema() -> kind_openai::GeneratedOpenAISchema {
                #schema_str.into()
            }
        }
    };

    TokenStream::from(expanded)
}

struct FieldInfo {
    name: String,
    schema: Value,
    required: bool,
    description: Option<String>,
}

fn get_description(attrs: &[Attribute]) -> Option<String> {
    let docs = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                match &attr.meta {
                    syn::Meta::NameValue(meta_name_value) => match &meta_name_value.value {
                        Expr::Lit(syn::ExprLit {
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
        .join("\n");
    if docs.is_empty() {
        None
    } else {
        Some(docs)
    }
}

fn get_field_type(ty: &Type) -> FieldSchema {
    match ty {
        Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            let type_name = segment.ident.to_string();
            match type_name.as_str() {
                "String" => FieldSchema {
                    schema: json!({"type": "string"}),
                    required: true,
                },
                "i32" | "i64" | "u32" | "u64" | "isize" | "usize" => FieldSchema {
                    schema: json!({"type": "integer"}),
                    required: true,
                },
                "f32" | "f64" => FieldSchema {
                    schema: json!({"type": "number"}),
                    required: true,
                },
                "bool" => FieldSchema {
                    schema: json!({"type": "boolean"}),
                    required: true,
                },
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_schema = get_field_type(inner_type);
                            return FieldSchema {
                                schema: json!({
                                    "type": "array",
                                    "items": inner_schema.schema
                                }),
                                required: true,
                            };
                        }
                    }
                    FieldSchema {
                        schema: json!({"type": "array", "items": {}}),
                        required: true,
                    }
                }
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_schema = get_field_type(inner_type);
                            let schema_with_null = match inner_schema.schema.clone() {
                                Value::Object(mut map) => {
                                    if let Some(Value::String(type_str)) = map.get("type") {
                                        map.insert(
                                            "type".to_string(),
                                            Value::Array(vec![
                                                Value::String(type_str.clone()),
                                                Value::String("null".to_string()),
                                            ]),
                                        );
                                        Value::Object(map)
                                    } else {
                                        json!({
                                            "anyOf": [
                                                inner_schema.schema,
                                                {"type": "null"}
                                            ]
                                        })
                                    }
                                }
                                _ => json!({
                                    "anyOf": [
                                        inner_schema.schema,
                                        {"type": "null"}
                                    ]
                                }),
                            };
                            return FieldSchema {
                                schema: schema_with_null,
                                required: false,
                            };
                        }
                    }
                    FieldSchema {
                        schema: json!({"type": "null"}),
                        required: false,
                    }
                }
                _ => panic!("Nested structs or enums are not supported"),
            }
        }
        _ => panic!("Unsupported type"),
    }
}
