mod enum_gen;
mod struct_gen;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use struct_gen::GenSegment;
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
    let repr = utils::has_repr_attr(&input.attrs)?;

    if utils::has_top_level_serde_attr(&input.attrs) {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "Top-level serde attrs are not supported",
        ));
    }

    match &input.data {
        Data::Struct(data) => {
            let tokens = struct_gen::handle_struct(data, name, description)?
                .into_iter()
                .map(|seg| match seg {
                    GenSegment::Quote(subordinate_get_schema_method_call) => quote! {
                        s.push_str(&#subordinate_get_schema_method_call);
                    },
                    GenSegment::StringLit(s) => quote! { s.push_str(&#s); },
                });

            Ok(quote! {
                impl ::kind_openai::OpenAISchema for #name {
                    fn openai_schema() -> ::kind_openai::GeneratedOpenAISchema {
                        use ::kind_openai::SubordinateOpenAISchema;
                        let mut s = ::std::string::String::new();
                        #(#tokens)*
                        s.into()
                    }
                }
            })
        }
        Data::Enum(data) => {
            let schema = serde_json::to_string(&enum_gen::handle_enum(data, repr, description)?)
                .map_err(|err| syn::Error::new_spanned(&input.ident, err.to_string()))?;

            Ok(quote! {
                impl ::kind_openai::SubordinateOpenAISchema for #name {
                    fn subordinate_openai_schema() -> &'static str {
                        #schema
                    }
                }
            })
        }
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "Only structs and enums with unit variants are supported",
        )),
    }
}
