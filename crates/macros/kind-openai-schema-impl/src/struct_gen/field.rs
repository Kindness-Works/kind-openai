use serde_json::Value;
use syn::{Field, Ident};

use crate::utils;

/// Our interpretable representation of a struct field with all necessary metadata
/// to turn it into a JSON schema object.
pub struct FieldInfo {
    pub name: String,
    pub schema: Value,
    pub required: bool,
    pub description: Option<String>,
}

impl<'a> TryFrom<FieldWithGuaranteedName<'a>> for FieldInfo {
    type Error = syn::Error;

    fn try_from(value: FieldWithGuaranteedName<'a>) -> Result<Self, Self::Error> {
        let field_schema = utils::get_field_type(value.ty())?;
        let description = utils::get_description(value.attrs());

        Ok(FieldInfo {
            name: value.name(),
            schema: field_schema.schema,
            required: field_schema.required,
            description,
        })
    }
}

pub struct FieldWithGuaranteedName<'a> {
    backing_field: &'a Field,
    name: FieldName,
}

enum FieldName {
    Anonymous(usize),
    Named(String),
}

impl From<&Ident> for FieldName {
    fn from(ident: &Ident) -> Self {
        Self::Named(ident.to_string())
    }
}

impl From<usize> for FieldName {
    fn from(index: usize) -> Self {
        Self::Anonymous(index)
    }
}

impl ToString for FieldName {
    fn to_string(&self) -> String {
        match self {
            FieldName::Anonymous(i) => i.to_string(),
            FieldName::Named(name) => name.clone(),
        }
    }
}

impl<'a> From<(usize, &'a Field)> for FieldWithGuaranteedName<'a> {
    fn from((i, field): (usize, &'a Field)) -> Self {
        let name = field
            .ident
            .as_ref()
            .map(Into::into)
            .unwrap_or_else(|| i.into());

        Self {
            backing_field: field,
            name,
        }
    }
}

impl<'a> FieldWithGuaranteedName<'a> {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn ty(&self) -> &syn::Type {
        &self.backing_field.ty
    }

    fn attrs(&self) -> &[syn::Attribute] {
        &self.backing_field.attrs
    }
}
