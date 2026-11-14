//! Source grammar data only; these controls issue no publication authority.
use super::*;
use crate::history::data::{
    CanonicalAuthoredRestoration, CanonicalAuthoredRetirement, CanonicalCoupledAuthoredPublication,
    CanonicalCoupledAuthoredSources as Sources, CommitId,
};

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

#[test]
fn corrective_source_codec_preserves_old_bytes_and_repeated_later_sources() {
    // An independent closed legacy grammar states the unchanged encoding.
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
        let bytes = rmp_serde::to_vec_named(&current).unwrap();
        assert_eq!(bytes, rmp_serde::to_vec_named(&old).unwrap());
        assert_eq!(rmp_serde::from_slice::<Sources>(&bytes).unwrap(), current);
    }
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&host);
    for source in [
        Sources::RetirementWithLaterSources {
            prefix: retirement,
            later: later.clone(),
        },
        Sources::RestorationWithLaterSources {
            prefix: restoration,
            later: later.clone(),
        },
    ] {
        let original = group(source);
        let bytes = rmp_serde::to_vec_named(&original).unwrap();
        let quote =
            super::super::coupled_publication::quote(BorrowedPayload(&bytes), &request).unwrap();
        let _scratch = request.reserve_retained_memory(quote.transient).unwrap();
        let memory = request.reserve_retained_memory(quote.retained).unwrap();
        let mut decoded: CanonicalCoupledAuthoredPublication =
            rmp_serde::from_slice(&bytes).unwrap();
        decoded._decoded_memory = Some(Arc::new(memory));
        assert_eq!(decoded.sources, original.sources);
        let actual = match &decoded.sources {
            Sources::RetirementWithLaterSources { later, .. }
            | Sources::RestorationWithLaterSources { later, .. } => later,
            _ => panic!("the new meaning was lost"),
        };
        assert_eq!(actual, &later);
        assert_eq!(rmp_serde::to_vec_named(&decoded).unwrap(), bytes);
    }
}

#[test]
fn corrective_source_quote_refuses_malformed_empty_missing_or_unknown_fields() {
    let (retirement, restoration) = prefixes();
    let scope = EntityId::new(PartitionId(1), 8, 1);
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&host);
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
            let bytes = rmp_serde::to_vec_named(&value).unwrap();
            assert!(
                super::super::coupled_publication::quote(BorrowedPayload(&bytes), &request)
                    .is_err(),
                "{tag} {defect}"
            );
        }
    }
}
