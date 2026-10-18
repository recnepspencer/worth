//! Render facts from the actual owning enum, reached through an audience route.
use super::configuration::FactRegion;
use std::path::Path;

pub(super) fn render(
    root: &Path,
    fact: &FactRegion,
    sources: &crate::facade_docs::resolution::CrateIndex,
) -> Result<String, String> {
    let item =
        crate::facade_docs::resolution::enum_definition(root, &fact.manifest, &fact.item, sources)?;
    let mut text = format!("<!-- Derived from `{}`; edit its source, then --write-guidance. -->\n\n| Variant | Shape |\n| --- | --- |\n", fact.item);
    let payload_name = |ty| {
        crate::facade_docs::resolution::payload_name(
            root,
            &fact.manifest,
            &fact.item,
            &item,
            ty,
            sources,
        )
    };
    for variant in &item.variants {
        let shape = match &variant.fields {
            syn::Fields::Unit => "unit".to_owned(),
            syn::Fields::Unnamed(fields) => format!(
                "tuple ({})",
                fields
                    .unnamed
                    .iter()
                    .map(|field| payload_name(&field.ty))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            ),
            syn::Fields::Named(fields) => format!(
                "named {{ {} }}",
                fields
                    .named
                    .iter()
                    .map(|field| Ok(format!(
                        "{}: {}",
                        field.ident.as_ref().ok_or("unnamed field")?,
                        payload_name(&field.ty)?
                    )))
                    .collect::<Result<Vec<_>, String>>()?
                    .join(", ")
            ),
        };
        text.push_str(&format!("| `{}` | {shape} |\n", variant.ident));
    }
    Ok(text)
}
