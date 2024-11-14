mod enum_gen;
mod struct_gen;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

/// Places an associated function on a struct that returns an `&'static str` containing its OpenAI-compatible JSON schema.
#[proc_macro_derive(OpenAISchema)]
pub fn openai_schema_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_openai_schema(&input) {
        Ok(expanded) => expanded.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_openai_schema(input: &DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let name = &input.ident;
    // this is the top-level docstring of the struct for the schema description.
    // individual field docstrings are also extracted.
    let description = utils::get_description(&input.attrs);

    let schema = match &input.data {
        Data::Struct(data) => struct_gen::handle_struct(data, name, description)?,
        Data::Enum(data) => enum_gen::handle_enum(data, name, description)?,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Only structs and enums with unit variants are supported",
            ));
        }
    };

    let schema_str = serde_json::to_string(&schema)
        .map_err(|err| syn::Error::new_spanned(&input.ident, err.to_string()))?;

    let expanded = quote! {
        impl kind_openai::OpenAISchema for #name {
            fn openai_schema() -> kind_openai::GeneratedOpenAISchema {
                #schema_str.into()
            }
        }
    };

    Ok(expanded)
}
