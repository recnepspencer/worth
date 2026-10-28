use syn::{Attribute, Ident, LitStr};

#[derive(Default)]
pub(super) struct Attributes {
    pub rename: Option<String>,
    pub tag: Option<String>,
    pub snake_case: bool,
    pub deny_unknown_fields: bool,
}
impl Attributes {
    pub fn parse(attributes: &[Attribute], container: bool) -> syn::Result<Self> {
        let mut result = Self::default();
        for attribute in attributes
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    if result.rename.is_some() { return Err(meta.error("duplicate Serde rename")); }
                    result.rename = Some(meta.value()?.parse::<LitStr>()?.value());
                } else if container && meta.path.is_ident("tag") {
                    if result.tag.is_some() { return Err(meta.error("duplicate Serde tag")); }
                    result.tag = Some(meta.value()?.parse::<LitStr>()?.value());
                } else if container && meta.path.is_ident("rename_all") {
                    if result.snake_case { return Err(meta.error("duplicate Serde rename_all")); }
                    let value: LitStr = meta.value()?.parse()?;
                    if value.value() != "snake_case" { return Err(meta.error("WorthValue supports rename_all = \"snake_case\"; other rename rules require a bounded owner codec")); }
                    result.snake_case = true;
                } else if container && meta.path.is_ident("deny_unknown_fields") {
                    if result.deny_unknown_fields { return Err(meta.error("duplicate deny_unknown_fields")); }
                    result.deny_unknown_fields = true;
                } else {
                    return Err(meta.error("unsupported Serde attribute for WorthValue; use supported literal rename/tag/snake_case/deny_unknown_fields or a bounded owner codec; defaults, skips, flattening and custom serializers are unsupported"));
                }
                Ok(())
            })?;
        }
        Ok(result)
    }
    pub fn name(&self, ident: &Ident, snake_case: bool) -> String {
        self.rename.clone().unwrap_or_else(|| {
            let name = ident.to_string().trim_start_matches("r#").to_owned();
            if !snake_case {
                return name;
            }
            let mut result = String::new();
            for (index, ch) in name.char_indices() {
                if ch.is_uppercase() && index > 0 {
                    result.push('_');
                }
                result.push(ch.to_ascii_lowercase());
            }
            result
        })
    }
}
/// Select the same canonical implementation through an audience facade.
pub(super) fn facade_path(attributes: &[syn::Attribute]) -> syn::Result<syn::Path> {
    let mut facade = None;
    for attribute in attributes
        .iter()
        .filter(|attr| attr.path().is_ident("worth_value"))
    {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("crate") || facade.is_some() {
                return Err(meta.error("WorthValue containers accept one `crate = \"facade::path\"`; delegates belong on named fields"));
            }
            let literal: syn::LitStr = meta.value()?.parse()?;
            facade = Some(literal.parse::<syn::Path>()?);
            Ok(())
        })?;
    }
    Ok(facade.unwrap_or_else(|| syn::parse_quote!(::worth_foundational::facade)))
}
