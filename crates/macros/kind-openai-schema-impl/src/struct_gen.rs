mod field;

use proc_macro2::Ident;
use serde_json::{json, Value};
use syn::{DataStruct, Fields};

pub fn handle_struct(
    data: &DataStruct,
    name: &Ident,
    description: Option<String>,
) -> Result<Value, syn::Error> {
    let field_infos = collect_field_infos(&data.fields)?;

    let mut properties = serde_json::Map::new();
    let mut required_fields = Vec::new();

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

        if field.required {
            required_fields.push(field.name.clone());
        }
    }

    Ok(json!({
        "name": name.to_string(),
        "description": description,
        "strict": true,
        "schema": {
            "type": "object",
            "properties": properties,
            "required": required_fields,
            "additionalProperties": false
        }
    }))
}

fn collect_field_infos(fields: &Fields) -> Result<Vec<field::FieldInfo>, syn::Error> {
    match fields {
        Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .enumerate()
            .map(Into::<field::FieldWithGuaranteedName>::into)
            .map(TryInto::try_into)
            .collect(),
        Fields::Unnamed(fields_unnamed) => fields_unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(Into::<field::FieldWithGuaranteedName>::into)
            .map(TryInto::try_into)
            .collect(),
        Fields::Unit => Ok(Vec::new()),
    }
}
