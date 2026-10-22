//! Task-index navigation and actual audience crate rustdoc entrances.
use super::configuration::{confined_existing, Configuration};
use pulldown_cmark::{Event, Parser, Tag};
use std::path::Path;

const REPOSITORY: &str = "https://github.com/recnepspencer/worth/blob/master/";
pub(super) fn validate(root: &Path, config: &Configuration) -> Result<(), String> {
    let Some(nav) = &config.navigation else {
        return Ok(());
    };
    let index = confined_existing(root, &nav.index)?;
    for guide in &nav.guides {
        let text = crate::source_documents::read_to_string(&root.join(guide))
            .map_err(|e| e.to_string())?;
        let linked = destinations(&text).into_iter().any(|link| {
            let path = link.split('#').next().unwrap();
            root.join(guide)
                .parent()
                .unwrap()
                .join(path)
                .canonicalize()
                .ok()
                .as_ref()
                == Some(&index)
        });
        if !linked {
            return Err(format!("{guide}: link the application capability index"));
        }
    }
    for source in &nav.crate_roots {
        let path = confined_existing(root, source)?;
        let text = crate::source_documents::read_to_string(&path).map_err(|e| e.to_string())?;
        let syntax = crate::source_syntax::parse_named(&path, &text).map_err(|e| e.to_string())?;
        let docs = syntax
            .attrs
            .iter()
            .filter_map(|attr| {
                let syn::Meta::NameValue(meta) = &attr.meta else {
                    return None;
                };
                if !meta.path.is_ident("doc") {
                    return None;
                }
                let syn::Expr::Lit(lit) = &meta.value else {
                    return None;
                };
                let syn::Lit::Str(value) = &lit.lit else {
                    return None;
                };
                Some(value.value())
            })
            .collect::<Vec<_>>()
            .join("\n");
        let links = destinations(&docs);
        for target in ["docs/build-an-application.md", nav.index.as_str()] {
            confined_existing(root, target)?;
            if !links
                .iter()
                .any(|link| link == &format!("{REPOSITORY}{target}"))
            {
                return Err(format!("{source}: crate rustdoc must link {target}"));
            }
        }
    }
    Ok(())
}
fn destinations(text: &str) -> Vec<String> {
    Parser::new(text)
        .filter_map(|event| match event {
            Event::Start(Tag::Link { dest_url, .. }) => Some(dest_url.into_string()),
            _ => None,
        })
        .collect()
}
