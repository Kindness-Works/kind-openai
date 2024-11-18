use serde_json::{json, Value};
use syn::{DataEnum, Expr, Fields, Lit};

use crate::utils;

pub fn handle_enum(
    data: &DataEnum,
    has_repr: bool,
    description: Option<String>,
) -> Result<Value, syn::Error> {
    let mut is_numeric_enum = true;
    let mut variant_values = Vec::new();

    for variant in &data.variants {
        match &variant.fields {
            Fields::Unit => {
                if let Some((
                    _,
                    Expr::Lit(syn::ExprLit {
                        lit: Lit::Int(lit_int),
                        ..
                    }),
                )) = &variant.discriminant
                {
                    if !has_repr {
                        return Err(syn::Error::new_spanned(
                            lit_int,
                            "repr attribute is required for enums with non-numeric variants.
NOTE: when using repr, ensure that you are using the `serde_repr` crate. It's impossible for us to detect that you are \
actually using that deserializer, so you will get runtime deserialization errors if not as we always generate \
numeric schemas when repr is detected.",
                        ));
                    }
                    let int_value = lit_int
                        .base10_parse::<i64>()
                        .map_err(|e| syn::Error::new_spanned(lit_int, e))?;
                    variant_values.push(json!(int_value));
                } else {
                    is_numeric_enum = false;
                    break;
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &variant.ident,
                    "Only enums with unit variants are supported",
                ));
            }
        }
    }

    let mut subordinate_schema = if is_numeric_enum && variant_values.len() == data.variants.len() {
        json!({
            "type": "number",
            "enum": variant_values
        })
    } else {
        let variant_names = data
            .variants
            .iter()
            .filter(|variant| !utils::get_serde_skip(&variant.attrs))
            .map(|variant| match &variant.fields {
                Fields::Unit => {
                    if utils::get_description(&variant.attrs).is_some() {
                        Err(syn::Error::new_spanned(
                            &variant.ident,
                            "Subordinate type descriptions should be located on the subordinate type itself and not on the field.",
                        ))
                    } else {
                        Ok(utils::get_serde_rename(&variant.attrs)
                            .unwrap_or_else(|| variant.ident.to_string()))
                    }
                },
                _ => unreachable!(), // we've have already checked non-unit
            })
            .collect::<Result<Vec<String>, syn::Error>>()?;

        json!({
            "type": "string",
            "enum": variant_names
        })
    };

    if let Some(description) = description {
        subordinate_schema["description"] = Value::String(description);
    }

    Ok(subordinate_schema)
}
