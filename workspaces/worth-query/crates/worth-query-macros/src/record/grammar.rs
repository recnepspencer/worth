//! Record keys, columns and the checks their generated names require.
use super::{DERIVE, MAXIMUM_FIELDS};
use crate::case::pascal_case;
use crate::grammar::{flag, member_name, option_inner, unknown_key, value, visit_keys, Once};
use std::collections::BTreeSet;
use syn::spanned::Spanned;
use syn::{DeriveInput, Field, FieldsNamed, Ident, LitStr, Type};

const STRUCT_KEYS: &[&str] = &["bindings", "aspect", "name", "fragment", "entity"];
const FIELD_KEYS: &[&str] = &["column", "equality", "read_only", "unique"];

pub(super) struct RecordDeclaration {
    pub(super) bindings: Type,
    pub(super) aspect: Ident,
    pub(super) aspect_name: LitStr,
    /// `(fragment, entity)` when column selection aliases are declared.
    pub(super) selections: Option<(Type, Type)>,
    pub(super) columns: Vec<Column>,
}

pub(super) struct Column {
    pub(super) member: Ident,
    pub(super) column: LitStr,
    /// The selection's value type: the field type, without `Option`.
    pub(super) stored: Type,
    pub(super) equality: bool,
    pub(super) read_only: bool,
    pub(super) unique: bool,
    /// The selection alias, named after the column, when selections are declared.
    pub(super) alias: Option<Ident>,
}

impl Column {
    fn alias_name(&self) -> syn::Result<Ident> {
        syn::parse_str::<Ident>(&self.column.value()).map_err(|_| {
            syn::Error::new(
                self.column.span(),
                format!(
                    "column `{}` is not a Rust type name, so its selection alias cannot be \
                     declared; omit `fragment`/`entity` and write the alias by hand",
                    self.column.value()
                ),
            )
        })
    }
}

impl RecordDeclaration {
    pub(super) fn parse(input: &DeriveInput, fields: &FieldsNamed) -> syn::Result<Self> {
        let mut bindings = Once::<Type>::new("bindings");
        let mut aspect = Once::<Ident>::new("aspect");
        let mut aspect_name = Once::<LitStr>::new("name");
        let mut fragment = Once::<Type>::new("fragment");
        let mut entity = Once::<Type>::new("entity");
        visit_keys(&input.attrs, |key, meta| match key {
            "bindings" => bindings.set(meta, value(meta)?),
            "aspect" => aspect.set(meta, value(meta)?),
            "name" => aspect_name.set(meta, value(meta)?),
            "fragment" => fragment.set(meta, value(meta)?),
            "entity" => entity.set(meta, value(meta)?),
            _ => Err(unknown_key(meta, DERIVE, STRUCT_KEYS)),
        })?;
        let selection_span = fragment.span().or(entity.span());
        let selections = match (fragment.optional(), entity.optional()) {
            (Some(fragment), Some(entity)) => Some((fragment, entity)),
            (None, None) => None,
            _ => {
                return Err(syn::Error::new(
                    selection_span.unwrap_or(input.ident.span()),
                    "`fragment` and `entity` declare the column selections together",
                ));
            }
        };
        let aspect = aspect.required(input.ident.span(), DERIVE)?;
        let aspect_name = aspect_name
            .optional()
            .unwrap_or_else(|| LitStr::new(&aspect.to_string(), aspect.span()));
        let mut columns = columns(input, fields)?;
        if selections.is_some() {
            for column in &mut columns {
                column.alias = Some(column.alias_name()?);
            }
        }
        Ok(Self {
            bindings: bindings.required(input.ident.span(), DERIVE)?,
            aspect,
            aspect_name,
            columns,
            selections,
        })
    }
}

fn columns(input: &DeriveInput, fields: &FieldsNamed) -> syn::Result<Vec<Column>> {
    if fields.named.is_empty() {
        return Err(syn::Error::new(
            input.ident.span(),
            "a record needs at least one field",
        ));
    }
    if let Some(extra) = fields.named.iter().nth(MAXIMUM_FIELDS) {
        return Err(syn::Error::new(
            extra.span(),
            format!(
                "a record holds at most {MAXIMUM_FIELDS} fields; move these into another aspect"
            ),
        ));
    }
    let mut written = BTreeSet::new();
    fields
        .named
        .iter()
        .map(|field| {
            let column = column(field)?;
            if !written.insert(column.column.value()) {
                return Err(syn::Error::new(
                    column.column.span(),
                    format!("duplicate column `{}`", column.column.value()),
                ));
            }
            Ok(column)
        })
        .collect()
}

fn column(field: &Field) -> syn::Result<Column> {
    let member = field.ident.clone().expect("named fields carry identifiers");
    let mut column = Once::<LitStr>::new("column");
    let (mut equality, mut read_only, mut unique) = (false, false, false);
    visit_keys(&field.attrs, |key, meta| {
        match key {
            "column" => column.set(meta, value(meta)?)?,
            "equality" => equality = flag(meta)?,
            "read_only" => read_only = flag(meta)?,
            "unique" => unique = flag(meta)?,
            _ => return Err(unknown_key(meta, DERIVE, FIELD_KEYS)),
        }
        Ok(())
    })?;
    let column = column
        .optional()
        .unwrap_or_else(|| LitStr::new(&pascal_case(&member_name(&member)), member.span()));
    Ok(Column {
        stored: option_inner(&field.ty).unwrap_or(&field.ty).clone(),
        member,
        column,
        equality,
        read_only,
        unique,
        alias: None,
    })
}
