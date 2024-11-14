use serde_json::{json, Value};
use syn::{DataEnum, Expr, Fields, Ident, Lit};

pub fn handle_enum(
    data: &DataEnum,
    name: &Ident,
    description: Option<String>,
) -> Result<Value, syn::Error> {
    let mut is_numeric_enum = true;
    let mut variant_values = Vec::new();

    for variant in &data.variants {
        match &variant.fields {
            Fields::Unit => {
                if let Some((_, expr)) = &variant.discriminant {
                    if let Expr::Lit(syn::ExprLit {
                        lit: Lit::Int(lit_int),
                        ..
                    }) = expr
                    {
                        let int_value = lit_int
                            .base10_parse::<i64>()
                            .map_err(|e| syn::Error::new_spanned(lit_int, e))?;
                        variant_values.push(json!(int_value));
                    } else {
                        is_numeric_enum = false;
                        break;
                    }
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

    if is_numeric_enum && variant_values.len() == data.variants.len() {
        Ok(json!({
            "name": name.to_string(),
            "description": description,
            "schema": {
                "type": "number",
                "enum": variant_values
            }
        }))
    } else {
        let variant_names = data
            .variants
            .iter()
            .map(|variant| match &variant.fields {
                Fields::Unit => variant.ident.to_string(),
                _ => unreachable!(), // we've have already checked non-unit
            })
            .collect::<Vec<String>>();

        Ok(json!({
            "name": name.to_string(),
            "description": description,
            "schema": {
                "type": "string",
                "enum": variant_names
            }
        }))
    }
}
