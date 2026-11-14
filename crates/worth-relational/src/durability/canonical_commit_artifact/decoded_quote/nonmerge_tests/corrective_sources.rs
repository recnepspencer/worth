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
    #[derive(serde::Serialize)]
    enum Legacy {
        Retirement(Arc<CanonicalAuthoredRetirement>),
        Restoration(Arc<CanonicalAuthoredRestoration>),
        Edit(EntityId),
        Stages(Vec<EntityId>),
    }
    let (retirement, restoration) = prefixes();
    let scope = EntityId::new(PartitionId(1), 8, 1);
    let later = vec![scope, scope];
    let mut values = Vec::new();
    for (old, current) in [
        (
            Legacy::Retirement(retirement.clone()),
            Sources::Retirement(retirement.clone()),
        ),
        (
            Legacy::Restoration(restoration.clone()),
            Sources::Restoration(restoration.clone()),
        ),
        (Legacy::Edit(scope), Sources::Edit(scope)),
        (
            Legacy::Stages(later.clone()),
            Sources::Stages(later.clone()),
        ),
    ] {
        assert_eq!(
            rmp_serde::to_vec_named(&current).unwrap(),
            rmp_serde::to_vec_named(&old).unwrap()
        );
        values.push(current);
    }
    values.extend([
        Sources::RetirementWithLaterSources {
            prefix: retirement,
            later: later.clone(),
        },
        Sources::RestorationWithLaterSources {
            prefix: restoration,
            later,
        },
    ]);
    values
        .into_iter()
        .map(|source| {
            let value = group(source);
            (rmp_serde::to_vec_named(&value).unwrap(), value.sources)
        })
        .collect()
}
pub(super) fn malformed_inputs() -> Vec<(String, Vec<u8>)> {
    let (retirement, restoration) = prefixes();
    let scope = EntityId::new(PartitionId(1), 8, 1);
    let mut cases = Vec::new();
    for (tag, source) in [
        (
            "RetirementWithLaterSources",
            Sources::RetirementWithLaterSources {
                prefix: retirement,
                later: vec![scope],
            },
        ),
        (
            "RestorationWithLaterSources",
            Sources::RestorationWithLaterSources {
                prefix: restoration,
                later: vec![scope],
            },
        ),
    ] {
        let original = serde_json::to_value(group(source)).unwrap();
        for defect in [
            "malformed",
            "empty",
            "missing-prefix",
            "missing-later",
            "unknown-field",
        ] {
            let mut value = original.clone();
            let fields = value["sources"][tag].as_object_mut().unwrap();
            match defect {
                "malformed" => {
                    fields.insert("later".into(), serde_json::json!("not an entity vector"));
                }
                "empty" => {
                    fields.insert("later".into(), serde_json::json!([]));
                }
                "missing-prefix" => {
                    fields.remove("prefix");
                }
                "missing-later" => {
                    fields.remove("later");
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
    cases
}
pub(super) fn malformed_stages() -> Vec<u8> {
    #[derive(serde::Serialize)]
    enum Source {
        Stages(&'static str),
    }
    #[derive(serde::Serialize)]
    struct Data {
        sources: Source,
        participants: Vec<()>,
    }
    rmp_serde::to_vec_named(&Data {
        sources: Source::Stages("not an entity vector"),
        participants: vec![],
    })
    .unwrap()
}
pub(super) fn unknown_source() -> Vec<u8> {
    #[derive(serde::Serialize)]
    enum Source {
        UnknownStages(Vec<EntityId>),
    }
    #[derive(serde::Serialize)]
    struct Data {
        sources: Source,
        participants: Vec<()>,
    }
    rmp_serde::to_vec_named(&Data {
        sources: Source::UnknownStages(vec![]),
        participants: vec![],
    })
    .unwrap()
}
