mod field;

use core::fmt;

use proc_macro2::Ident;
use quote::quote;
use serde::Serialize;
use serde_json::Value;
use syn::{DataStruct, Fields};

use crate::utils::Schema;

pub enum GenSegment {
    StringLit(String),
    Quote(proc_macro2::TokenStream),
}

pub struct JsonField<'a, T>(&'a T);

impl<'a, T> fmt::Display for JsonField<'a, T>
where
    T: Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(self.0).unwrap())?;
        Ok(())
    }
}

/*
this implementation is a wee bit insane and was done fairly quickly to unblock some feature work that needed enums.
i'll do my best to explain how this works in this top level comment since inline comments won't do muc good here.

openai schemas have a rule where the top level object MUST be a `"type": "object"` which in and of itself cannot be an
enum. enums are supported, but they're supported as subfields of said struct. in rust, proc macros are scoped to the
struct that they are placed upon, so there's no way to have some kind of global state for a proc macro. what this means
is that we can't do something like the following (psuedocode):

#[derive(OpenAISchema)]
struct MyStruct {
    my_enum: MyEnum,
}

#[derive(OpenAISchema)]
enum MyEnum {
    X,
    Y,
}

fn the_proc_macro_source_code() {
    let fields = get_fields();

    for field in fields {
        match field.type {
            ... all the primitives + Vec, Option, etc.
            UserType(type) => type::some_method_that_is_only_visible_at_compile_time() // impossible
        }
    }
}

instead, we have to have the derive on top of an enum inject some method onto the enum that contains the subordinate schema
definition, and then, on the method injected on the root struct, we need to put json literals representing the schema where
possible, and then crunch subordinate types by instructing the struct's injected method to look up said subordinate type on
a given type.

to do this, the following function works like such:

1. build up the root schema object consisting of the name, description, and strict value
2. push a PARTIAL of the schema object that goes up until the properties (we can't actually build a struct and auto serialize
it because there is no way to represent a quoted string in a serialize object)
3. for each field, check if it can be statically represented...
    a. if it can be statically represented, just push the json literal representing the serialized schema field value
    b. if it can't push up an quote containing an expression that returns a static string containing the schema
4. clean up by closing the opened json strings

there is probably a more elegant way to do this, but as far as i know it's impossible to do this in a way where we are building
an object, then serializing it but omitting quotes in the correct order. i suppose we could do this by serializing then jumping
to fixed indexes, but that seems much hackier than what i'm doing below :shrug:
*/
pub fn handle_struct(
    data: &DataStruct,
    name: &Ident,
    description: Option<String>,
) -> Result<Vec<GenSegment>, syn::Error> {
    let mut segments = Vec::new();

    // the root of the schema that contains a non-delimited object that contains the properties
    segments.push(GenSegment::StringLit(format!(
        r#"{{"name":{},"description":{},"strict":true,"schema":{{"type":"object","additionalProperties":false,"properties":{{"#,
        JsonField(&name.to_string()),
        JsonField(&description)
    )));

    let mut required_fields = Vec::new();

    for field in collect_field_infos(&data.fields)?.into_iter().flatten() {
        segments.push(GenSegment::StringLit(format!(
            "{}:",
            // serialize the field name as a string since it will hopefully be a valid json key
            JsonField(&field.name)
        )));

        match field.schema {
            Schema::Inlined(mut schema) => {
                // modify the provided schema to contain the description since the codepoint where the schema object is made
                // does not have access the any kind of description data
                if let (Some(description), Some(obj)) =
                    (field.description.as_ref(), schema.as_object_mut())
                {
                    obj.insert(
                        "description".to_string(),
                        Value::String(description.clone()),
                    );
                }

                segments.push(GenSegment::StringLit(JsonField(&schema).to_string()));
            }
            Schema::Subordinate(ty_name) => {
                if field.description.is_some() {
                    return Err(syn::Error::new_spanned(
                        &field.name,
                        "Subordinate type descriptions should be located on the subordinate type itself and not on the field.",
                    ));
                }
                segments.push(GenSegment::Quote(quote! {
                    #ty_name::subordinate_openai_schema()
                }))
            }
        }

        if field.required {
            required_fields.push(field.name);
        }

        segments.push(GenSegment::StringLit(",".to_string()));
    }

    // remove the trailing comma
    segments.pop();

    // closing the root object containing the key value pairs pushed above
    segments.push(GenSegment::StringLit("},".to_string()));
    // push the required fields and close the entire object
    segments.push(GenSegment::StringLit(format!(
        r#""required":{}}}}}"#,
        JsonField(&required_fields)
    )));

    Ok(segments)
}

fn collect_field_infos(fields: &Fields) -> Result<Vec<Option<field::FieldInfo>>, syn::Error> {
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
