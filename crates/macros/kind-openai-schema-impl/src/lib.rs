use proc_macro::TokenStream;
use quote::quote;
use serde_json::{json, Value};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type,
};

#[proc_macro_derive(OpenAISchema)]
pub fn openai_schema_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let description = get_description(&input.attrs);

    let properties = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| {
                    let field_name = f.ident.as_ref().unwrap().to_string();
                    let field_type = get_field_type(&f.ty);
                    (field_name, field_type)
                })
                .collect::<serde_json::Map<String, Value>>(),
            Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let field_name = i.to_string();
                    let field_type = get_field_type(&f.ty);
                    (field_name, field_type)
                })
                .collect::<serde_json::Map<String, Value>>(),
            Fields::Unit => serde_json::Map::new(),
        },
        _ => panic!("Only structs are supported"),
    };

    let required: Vec<String> = properties.keys().cloned().collect();

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

fn get_description(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("doc"))
        .map(|attr| attr.parse_args::<syn::LitStr>().unwrap().value())
        .unwrap_or_default()
}

fn get_field_type(ty: &Type) -> Value {
    match ty {
        Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            let type_name = segment.ident.to_string();
            match type_name.as_str() {
                "String" => json!({"type": "string"}),
                "i32" | "i64" | "f32" | "f64" => json!({"type": "number"}),
                "bool" => json!({"type": "boolean"}),
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            return json!({
                                "type": "array",
                                "items": get_field_type(inner_type)
                            });
                        }
                    }
                    json!({"type": "array", "items": {}})
                }
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_schema = get_field_type(inner_type);
                            return json!({
                                "anyOf": [
                                    inner_schema,
                                    {"type": "null"}
                                ]
                            });
                        }
                    }
                    json!({"anyOf": [{"type": "null"}]})
                }
                _ => json!({"type": "object"}), // assume custom types are objects
            }
        }
        _ => panic!("Unsupported type"),
    }
}
