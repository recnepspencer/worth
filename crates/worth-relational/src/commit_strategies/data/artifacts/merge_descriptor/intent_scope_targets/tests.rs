//! Prior canonical locator codec and descriptor grammar remain byte-identical.
use super::super::*;
use worth_foundational::facade::{AspectKey, FieldKey};

#[derive(Serialize)]
struct LegacyDescriptor {
    strategy_id: CommitStrategyId,
    descriptor_digest: CommitStrategyDescriptorDigest,
    semantic_name: CommitStrategySemanticName,
    family_name: CommitStrategyFamilyName,
    version: CommitStrategyVersion,
    intent_name: StrategyIntentName,
    intent_scope_digest: StrategyIntentScopeDigest,
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator_arc_slice")]
    intent_scope_targets: Arc<[AspectFieldLocator]>,
    merge_semantics: StrategyMergeSemantics,
    lowering_summary_digest: worth_foundational::CanonicalDigestId,
}

#[test]
fn strategy_scope_targets_preserve_prior_named_compact_wire_and_ordinary_clone() {
    let locator = |aspect, field| {
        crate::transactions::data::planned_single_field_locator(
            AspectKey::new(aspect).unwrap(),
            FieldKey::new(field).unwrap(),
        )
    };
    for targets in [
        vec![],
        vec![
            locator("deployment.desired", "replicas"),
            locator("deployment.observed", "status"),
        ],
    ] {
        let targets: Arc<[AspectFieldLocator]> = targets.into();
        let value = StrategyMergeDescriptor {
            strategy_id: CommitStrategyId(41),
            descriptor_digest: CommitStrategyDescriptorDigest([2; 32]),
            semantic_name: CommitStrategySemanticName::new("strategy.λ"),
            family_name: CommitStrategyFamilyName::new("strategy.intent"),
            version: CommitStrategyVersion::new(1, 2),
            intent_name: StrategyIntentName::new("reconcile"),
            intent_scope_digest: StrategyIntentScopeDigest::new([3; 32]),
            intent_scope_targets: targets.clone(),
            merge_semantics: StrategyMergeSemantics::new(
                StrategyMergeConflictClass::IntentReconciliation,
                true,
                true,
            ),
            lowering_summary_digest: worth_foundational::CanonicalDigestId::new([4; 32]),
            intent_scope_targets_memory: None,
        };
        let legacy = LegacyDescriptor {
            strategy_id: value.strategy_id,
            descriptor_digest: value.descriptor_digest,
            semantic_name: value.semantic_name.clone(),
            family_name: value.family_name.clone(),
            version: value.version,
            intent_name: value.intent_name.clone(),
            intent_scope_digest: value.intent_scope_digest,
            intent_scope_targets: targets,
            merge_semantics: value.merge_semantics,
            lowering_summary_digest: value.lowering_summary_digest,
        };
        assert!(value.intent_scope_targets_memory.is_none());
        let clone = value.clone();
        assert!(clone.intent_scope_targets_memory.is_none());
        assert!(Arc::ptr_eq(
            &clone.intent_scope_targets,
            &value.intent_scope_targets
        ));
        assert_eq!(clone, value);
        assert_eq!(format!("{clone:?}"), format!("{value:?}"));
        let named = rmp_serde::to_vec_named(&legacy).unwrap();
        let compact = rmp_serde::to_vec(&legacy).unwrap();
        assert_eq!(rmp_serde::to_vec_named(&value).unwrap(), named);
        assert_eq!(rmp_serde::to_vec(&value).unwrap(), compact);
        for wire in [named, compact] {
            let decoded: StrategyMergeDescriptor = rmp_serde::from_slice(&wire).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(decoded.intent_scope_targets(), value.intent_scope_targets());
            assert!(decoded.intent_scope_targets_memory.is_none());
        }
    }
}
