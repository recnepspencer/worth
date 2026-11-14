//! Pure grammar data for the existing decoded-owner unit host; no runtime roots.
use crate::history::data::{
    CanonicalAuthoredRestoration, CanonicalAuthoredRetirement, CanonicalCoupledAuthoredPublication,
    CanonicalCoupledAuthoredSources as Sources, CommitId,
};
use crate::identity::data::{EntityId, PartitionId};
use std::sync::Arc;
fn prefixes() -> (
    Arc<CanonicalAuthoredRetirement>,
    Arc<CanonicalAuthoredRestoration>,
) {
    let old = EntityId::new(PartitionId(1), 2, 1);
    let fresh = EntityId::new(PartitionId(1), 3, 2);
    (
        Arc::new(CanonicalAuthoredRetirement {
            anchor: old,
            owned_members: false,
            birth: CommitId(1),
            co_anchors: vec![],
            publications: vec![],
            _decoded_memory: None,
        }),
        Arc::new(CanonicalAuthoredRestoration {
            retirement: CommitId(2),
            original_anchor: old,
            anchor: fresh,
            owned_members: false,
            co_anchors: vec![],
            entities: vec![(old, fresh)],
            relations: vec![],
            publications: vec![],
            _decoded_memory: None,
        }),
    )
}
fn group(sources: Sources) -> CanonicalCoupledAuthoredPublication {
    CanonicalCoupledAuthoredPublication {
        sources,
        participants: vec![],
        live_inputs: Default::default(),
        _decoded_memory: None,
    }
}
pub(super) fn roundtrips() -> Vec<(Vec<u8>, Sources)> {
    let (retirement, restoration) = prefixes();
    let scope = EntityId::new(PartitionId(1), 8, 1);
    let mut values = Vec::new();
    for later in [vec![], vec![scope], vec![scope, scope]] {
        values.extend([
            Sources::Edit {
                sources: std::iter::once(scope)
                    .chain(later.iter().copied())
                    .collect(),
            },
            Sources::Retirement {
                prefix: retirement.clone(),
                later: later.clone(),
            },
            Sources::Restoration {
                prefix: restoration.clone(),
                later,
            },
        ]);
    }
    values
        .into_iter()
        .map(|source| {
            let value = group(source);
            (rmp_serde::to_vec_named(&value).unwrap(), value.sources)
        })
        .collect()
}
pub(super) fn malformed_inputs() -> Vec<(String, Vec<u8>)> {
    let mut cases = Vec::new();
    for (_, source) in roundtrips().into_iter().take(3) {
        let original = serde_json::to_value(group(source)).unwrap();
        let tag = original["sources"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap();
        let sequence = if tag == "Edit" { "sources" } else { "later" };
        for defect in [
            "malformed",
            "missing-sequence",
            "missing-prefix",
            "unknown-field",
        ] {
            if defect == "missing-prefix" && tag == "Edit" {
                continue;
            }
            let mut value = original.clone();
            let fields = value["sources"][tag].as_object_mut().unwrap();
            match defect {
                "malformed" => {
                    fields.insert(sequence.into(), serde_json::json!("not an entity vector"));
                }
                "missing-sequence" => {
                    fields.remove(sequence);
                }
                "missing-prefix" => {
                    fields.remove("prefix");
                }
                "unknown-field" => {
                    fields.insert("extra".into(), serde_json::json!(0));
                }
                _ => unreachable!(),
            }
            cases.push((
                format!("{tag} {defect}"),
                rmp_serde::to_vec_named(&value).unwrap(),
            ));
        }
    }
    for (label, sources) in [
        (
            "missing edit prefix",
            serde_json::json!({"Edit":{"sources":[]}}),
        ),
        ("unknown semantic kind", serde_json::json!({"Unknown":{}})),
    ] {
        let value = serde_json::json!({"sources":sources,"participants":[]});
        cases.push((label.into(), rmp_serde::to_vec_named(&value).unwrap()));
    }
    cases
}
