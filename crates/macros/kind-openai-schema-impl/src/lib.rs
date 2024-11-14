use proc_macro::TokenStream;
use quote::quote;
use serde_json::{json, Value};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type,
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

    let field_infos = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| {
                    let field_name = f.ident.as_ref().unwrap().to_string();
                    let field_schema = get_field_type(&f.ty);
                    FieldInfo {
                        name: field_name,
                        schema: field_schema.schema,
                        required: field_schema.required,
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
                    FieldInfo {
                        name: field_name,
                        schema: field_schema.schema,
                        required: field_schema.required,
                    }
                })
                .collect::<Vec<FieldInfo>>(),
            Fields::Unit => Vec::new(),
        },
        _ => panic!("Only structs are supported"),
    };

    let mut properties = serde_json::Map::new();
    for field in &field_infos {
        properties.insert(field.name.clone(), field.schema.clone());
    }

    let required: Vec<String> = field_infos
        .iter()
        .filter(|f| f.required)
        .map(|f| f.name.clone())
        .collect();

    let schema = json!({
        "name": name.to_string(),
        "description": description,
        "strict": true,
        "schema": {
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false
        }
    });

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
}

fn get_description(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("doc"))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value())
        .unwrap_or_default()
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
                "i32" | "i64" | "f32" | "f64" => FieldSchema {
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
                _ => FieldSchema {
                    schema: json!({"type": "object"}),
                    required: true,
                }, // assume custom types are objects
            }
        }
        _ => panic!("Unsupported type"),
    }
}
