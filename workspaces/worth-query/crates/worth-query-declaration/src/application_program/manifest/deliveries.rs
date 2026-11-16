//! Canonical delivery definitions are quoted once; actions reference meaning.
//! These local indices describe this manifest, never runtime types or authority.
use super::super::{semantic_encoding::framed_parts, ApplicationActionDeclaration};
use super::framed_record;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn append(records: &mut Vec<String>, actions: &[ApplicationActionDeclaration]) {
    let connections = actions
        .iter()
        .filter_map(|action| action.delivery()?.output())
        .flat_map(|delivery| delivery.canonical_connections())
        .collect::<BTreeSet<_>>();
    let connection_indices = connections
        .iter()
        .enumerate()
        .map(|(index, meaning)| (meaning.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    records.extend(connections.iter().enumerate().map(|(index, meaning)| {
        framed_record(
            "delivery-connection-v1",
            [("index", index.to_string()), ("meaning", meaning.clone())],
        )
    }));

    let meanings = actions
        .iter()
        .filter_map(|action| {
            let delivery = action.delivery()?;
            let meaning = if let Some(output) = delivery.output() {
                // Keep repetitions in the graph. Sharing the quoted values
                // must not turn a repeated member into a different declaration.
                let references = output
                    .canonical_connections()
                    .iter()
                    .map(|meaning| connection_indices[meaning.as_str()].to_string())
                    .collect::<Vec<_>>();
                let graph =
                    framed_parts(&references.iter().map(String::as_str).collect::<Vec<_>>());
                output.canonical_meaning_with_graph(&graph)
            } else {
                delivery.canonical_meaning()
            };
            Some((action.semantic_subject(), meaning))
        })
        .collect::<Vec<_>>();
    let definitions = meanings
        .iter()
        .map(|(_, meaning)| meaning.as_str())
        .collect::<BTreeSet<_>>();
    let definition_indices = definitions
        .iter()
        .enumerate()
        .map(|(index, meaning)| (*meaning, index))
        .collect::<BTreeMap<_, _>>();
    records.extend(definitions.iter().enumerate().map(|(index, meaning)| {
        framed_record(
            "delivery-definition-v1",
            [
                ("index", index.to_string()),
                ("meaning", (*meaning).to_owned()),
            ],
        )
    }));
    records.extend(meanings.iter().map(|(action, meaning)| {
        framed_record(
            "delivery",
            [
                ("action", action.clone()),
                (
                    "definition",
                    definition_indices[meaning.as_str()].to_string(),
                ),
            ],
        )
    }));
}
