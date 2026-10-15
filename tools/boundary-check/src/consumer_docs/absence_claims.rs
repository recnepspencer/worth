//! Explicit absence assertions and canonical references, never guessed prose meaning.
use super::configuration::Configuration;
use crate::facade_docs::resolution::{export_knowledge, Knowledge};
use pulldown_cmark::{Event, Parser, Tag};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn validate(root: &Path, config: &Configuration) -> Result<(), String> {
    let claims = config
        .absence_claims
        .iter()
        .map(|claim| (claim.id.as_str(), claim))
        .collect::<BTreeMap<_, _>>();
    let mut canonical = BTreeSet::new();
    let mut references = Vec::new();
    for document in &config.documents {
        let text = crate::source_documents::read_to_string(&root.join(document))
            .map_err(|e| e.to_string())?;
        for prefix in ["<!-- facade-absence:", "<!-- known-gap:"] {
            for (marker, range) in super::generated_regions::markers(&text, prefix)? {
                let definition = prefix == "<!-- facade-absence:";
                let id = marker
                    .strip_prefix(&format!("{prefix} "))
                    .and_then(|s| s.strip_suffix(" -->"))
                    .filter(|s| super::configuration::valid_id(s))
                    .ok_or("invalid absence marker grammar")?;
                if definition {
                    if config.navigation.as_ref().map(|nav| nav.index.as_str())
                        != Some(document.as_str())
                    {
                        return Err("canonical absence claims belong in capability index".into());
                    }
                    if !canonical.insert(id.to_owned()) {
                        return Err(format!("duplicate canonical claim {id}"));
                    }
                    let heading = format!("### Gap: {id}");
                    if text[range.end..].trim_start().lines().next() != Some(heading.as_str())
                        || Parser::new(&text)
                            .into_offset_iter()
                            .filter(|(event, span)| {
                                matches!(event, Event::Start(Tag::Heading { .. }))
                                    && text[span.clone()].trim() == heading
                            })
                            .count()
                            != 1
                        || !super::links::anchors(&text).contains(&format!("gap-{id}"))
                    {
                        return Err(format!(
                            "{id}: canonical claim requires a unique adjacent Gap heading"
                        ));
                    }
                    let claim = claims
                        .get(id)
                        .ok_or_else(|| format!("unqualified native absence claim {id}"))?;
                    match export_knowledge(
                        root,
                        &claim.manifest,
                        &claim.container,
                        &claim.name,
                        claim.kind,
                    ) {
                        Knowledge::ProvenAbsent => {}
                        Knowledge::Present => {
                            return Err(format!(
                                "{id}: source is present; remove stale absence claim"
                            ))
                        }
                        Knowledge::Unsupported(reason) => {
                            return Err(format!("{id}: unsupported absence proof: {reason}"))
                        }
                    }
                } else {
                    let next = text[range.end..].trim_start().lines().next().unwrap_or("");
                    let index = config
                        .navigation
                        .as_ref()
                        .ok_or("gap reference has no index")?;
                    let index_path = root
                        .join(&index.index)
                        .canonicalize()
                        .map_err(|e| e.to_string())?;
                    let linked = Parser::new(next).any(|event| {
                        let Event::Start(Tag::Link { dest_url, .. }) = event else {
                            return false;
                        };
                        let Some(path) = dest_url.strip_suffix(&format!("#gap-{id}")) else {
                            return false;
                        };
                        root.join(document)
                            .parent()
                            .unwrap()
                            .join(path)
                            .canonicalize()
                            .ok()
                            .as_ref()
                            == Some(&index_path)
                    });
                    if !linked {
                        return Err(format!(
                            "{id}: known-gap marker requires an adjacent canonical link"
                        ));
                    }
                    references.push(id.to_owned());
                }
            }
        }
    }
    for id in references {
        if !canonical.contains(&id) {
            return Err(format!("known-gap {id} has no checked canonical claim"));
        }
    }
    for id in claims.keys() {
        if !canonical.contains(*id) {
            return Err(format!("unused absence binding {id}"));
        }
    }
    Ok(())
}
