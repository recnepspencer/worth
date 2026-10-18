//! Canonical prohibition routing and actual audience crate rustdoc entrances.
use super::configuration::{confined_existing, Configuration};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::path::Path;

const REPOSITORY: &str = "https://github.com/recnepspencer/worth/blob/master/";
pub(super) fn validate(root: &Path, config: &Configuration) -> Result<(), String> {
    let Some(nav) = &config.navigation else {
        return Ok(());
    };
    let index = confined_existing(root, &nav.index)?;
    let text = crate::source_documents::read_to_string(&index).map_err(|e| e.to_string())?;
    let markers = super::generated_regions::markers(&text, "<!-- consumer-prohibitions")?;
    let [(begin, start), (end, finish)] = markers.as_slice() else {
        return Err("one active canonical prohibition region is required".into());
    };
    if begin != "<!-- consumer-prohibitions begin -->"
        || end != "<!-- consumer-prohibitions end -->"
        || start.end > finish.start
        || !super::links::anchors(&text).contains("consumer-prohibitions")
    {
        return Err("invalid canonical prohibition boundaries/anchor".into());
    }
    let region = &text[start.end..finish.start];
    let mut ids = std::collections::BTreeSet::new();
    let mut heading = None;
    for event in Parser::new(region) {
        match event {
            Event::Start(Tag::Heading { .. }) => heading = Some(String::new()),
            Event::Text(value) | Event::Code(value) if heading.is_some() => {
                heading.as_mut().unwrap().push_str(&value)
            }
            Event::End(TagEnd::Heading(_)) => {
                let title = heading.take().unwrap();
                if title.starts_with("Prohibition: ") {
                    let slug = super::links::anchors(&format!("### {title}\n"))
                        .into_iter()
                        .next()
                        .ok_or("missing prohibition ID")?;
                    if !ids.insert(slug) {
                        return Err("duplicate canonical prohibition ID".into());
                    }
                }
            }
            _ => {}
        }
    }
    if ids.is_empty() {
        return Err("canonical prohibition entries are missing".into());
    }
    for guide in &nav.guides {
        let text = crate::source_documents::read_to_string(&root.join(guide))
            .map_err(|e| e.to_string())?;
        let linked = destinations(&text).into_iter().any(|link| {
            let Some(path) = link.strip_suffix("#consumer-prohibitions") else {
                return false;
            };
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
            return Err(format!("{guide}: link the canonical consumer prohibitions"));
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
