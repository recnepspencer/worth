use std::sync::Arc;

use super::current_topology_rewire_admission_policy;
use super::decisions::{
    aggregate_record_resolution, decision_boundary_for_aspect, ownership_surface_for_policies,
    summarize_policy_records,
};
use crate::identity::data::{EntityId, PartitionId};
use crate::merge::data::{
    AspectMergePolicyKind, CustomMergePolicyIdentity, DeletionMergeClass, MergeConflictClass,
    MergeManualResolutionClass, MergePolicyDecisionBoundary, MergePolicyOwnershipClass,
    MergePolicyOwnershipSurface, MergePolicyProofBoundary, MergePolicyRejectClass,
    MergePolicyResolutionRecord, ResolvedAspectMergePolicy, TopologyRewireAdmissionPolicy,
};
use crate::transactions::data::RecordRef;
use worth_foundational::facade::AspectKey;

#[test]
fn topology_rewire_policy_is_explicitly_fail_closed_in_7d() {
    assert_eq!(
        current_topology_rewire_admission_policy(),
        TopologyRewireAdmissionPolicy::AlwaysEscalateToTopologyRegion
    );
}

#[test]
fn deleted_on_both_sides_without_aspect_rows_is_auto_resolved() {
    assert_eq!(
        aggregate_record_resolution(
            MergeConflictClass::Deletion(DeletionMergeClass::DeletedOnBothSides),
            &[],
        ),
        MergePolicyDecisionBoundary::AutoResolved
    );
}

#[test]
fn one_sided_deletion_stays_manual_even_when_aspect_rows_auto_resolve() {
    let aspects = [crate::merge::data::AspectPolicyResolutionRecord {
        aspect_key: AspectKey::new("name").unwrap(),
        comparison: crate::merge::data::AspectComparisonState::Equal,
        applied_policy: None,
        decision_boundary: MergePolicyDecisionBoundary::AutoResolved,
        resolved_value_strategy: None,
    }];

    assert_eq!(
        aggregate_record_resolution(
            MergeConflictClass::Deletion(DeletionMergeClass::SourceLiveTargetDeleted),
            &aspects,
        ),
        MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::GenericRuntimeConflict,
        }
    );
}

#[test]
fn ownership_class_distinguishes_runtime_and_custom_policies() {
    assert_eq!(
        AspectMergePolicyKind::PreferRicher.ownership_class(),
        MergePolicyOwnershipClass::RuntimeBuiltIn
    );
    assert_eq!(
        AspectMergePolicyKind::Custom(CustomMergePolicyIdentity {
            name: Arc::<str>::from("domain").into(),
            semantic_version: 1,
        })
        .ownership_class(),
        MergePolicyOwnershipClass::CustomPolicy
    );
}

#[test]
fn ownership_surface_reports_custom_policy_participation() {
    let runtime_only = [ResolvedAspectMergePolicy {
        aspect_key: AspectKey::new("name").unwrap(),
        policy: AspectMergePolicyKind::PreferRicher,
    }];
    let custom = [ResolvedAspectMergePolicy {
        aspect_key: AspectKey::new("name").unwrap(),
        policy: AspectMergePolicyKind::Custom(CustomMergePolicyIdentity {
            name: Arc::<str>::from("domain").into(),
            semantic_version: 1,
        }),
    }];

    assert_eq!(
        ownership_surface_for_policies(&runtime_only),
        MergePolicyOwnershipSurface::RuntimeOnly
    );
    assert_eq!(
        ownership_surface_for_policies(&custom),
        MergePolicyOwnershipSurface::ContainsCustomPolicy
    );
}

#[test]
fn policy_summary_reports_runtime_only_vs_custom_record_counts() {
    let records = Arc::from(vec![
        MergePolicyResolutionRecord {
            record: RecordRef::Entity(EntityId::new(PartitionId::main(), 1, 1)),
            target_record: None,
            classification: MergeConflictClass::ExactSharedTruth,
            aspect_resolutions: Arc::from(Vec::new()),
            applied_policies: Arc::from(Vec::new()),
            proof_boundary: MergePolicyProofBoundary {
                ownership_surface: MergePolicyOwnershipSurface::RuntimeOnly,
                decision_boundary: MergePolicyDecisionBoundary::AutoResolved,
            },
        },
        MergePolicyResolutionRecord {
            record: RecordRef::Entity(EntityId::new(PartitionId::main(), 2, 1)),
            target_record: None,
            classification: MergeConflictClass::SchemaDeclaredCorrespondence,
            aspect_resolutions: Arc::from(Vec::new()),
            applied_policies: Arc::from(Vec::new()),
            proof_boundary: MergePolicyProofBoundary {
                ownership_surface: MergePolicyOwnershipSurface::ContainsCustomPolicy,
                decision_boundary: MergePolicyDecisionBoundary::RequiresManualResolution {
                    class: MergeManualResolutionClass::GenericRuntimeConflict,
                },
            },
        },
    ]);

    let summary = summarize_policy_records(records);
    assert_eq!(summary.runtime_only_record_count, 1);
    assert_eq!(summary.custom_policy_record_count, 1);
}

#[test]
fn aggregate_record_resolution_preserves_specific_manual_resolution_class() {
    let aspects = [crate::merge::data::AspectPolicyResolutionRecord {
        aspect_key: AspectKey::new("name").unwrap(),
        comparison: crate::merge::data::AspectComparisonState::Unavailable,
        applied_policy: None,
        decision_boundary: MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::MissingVisibleState,
        },
        resolved_value_strategy: None,
    }];

    assert_eq!(
        aggregate_record_resolution(MergeConflictClass::DivergentVisibleState, &aspects),
        MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::MissingVisibleState,
        }
    );
}

#[test]
fn aggregate_record_resolution_marks_mixed_manual_resolution_classes_explicitly() {
    let aspects = [
        crate::merge::data::AspectPolicyResolutionRecord {
            aspect_key: AspectKey::new("name").unwrap(),
            comparison: crate::merge::data::AspectComparisonState::Unavailable,
            applied_policy: None,
            decision_boundary: MergePolicyDecisionBoundary::RequiresManualResolution {
                class: MergeManualResolutionClass::MissingVisibleState,
            },
            resolved_value_strategy: None,
        },
        crate::merge::data::AspectPolicyResolutionRecord {
            aspect_key: AspectKey::new("other").unwrap(),
            comparison: crate::merge::data::AspectComparisonState::TargetOnly,
            applied_policy: None,
            decision_boundary: MergePolicyDecisionBoundary::RequiresManualResolution {
                class: MergeManualResolutionClass::UnvalidatedSchemaCorrespondence,
            },
            resolved_value_strategy: None,
        },
    ];

    assert_eq!(
        aggregate_record_resolution(MergeConflictClass::SchemaDeclaredCorrespondence, &aspects),
        MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::MixedAspectManualResolution,
        }
    );
}

#[test]
fn aggregate_record_resolution_preserves_specific_reject_class() {
    let aspects = [crate::merge::data::AspectPolicyResolutionRecord {
        aspect_key: AspectKey::new("name").unwrap(),
        comparison: crate::merge::data::AspectComparisonState::Divergent,
        applied_policy: Some(AspectMergePolicyKind::FailOnConflict),
        decision_boundary: MergePolicyDecisionBoundary::Reject {
            class: MergePolicyRejectClass::BuiltInFailOnConflict,
        },
        resolved_value_strategy: None,
    }];

    assert_eq!(
        aggregate_record_resolution(MergeConflictClass::SchemaDeclaredCorrespondence, &aspects),
        MergePolicyDecisionBoundary::Reject {
            class: MergePolicyRejectClass::BuiltInFailOnConflict,
        }
    );
}

#[test]
fn aggregate_record_resolution_preserves_strategy_conflict_over_generic_aspect_manual_class() {
    let aspects = [crate::merge::data::AspectPolicyResolutionRecord {
        aspect_key: AspectKey::new("replicas").unwrap(),
        comparison: crate::merge::data::AspectComparisonState::Divergent,
        applied_policy: None,
        decision_boundary: MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::GenericRuntimeConflict,
        },
        resolved_value_strategy: None,
    }];

    assert_eq!(
        aggregate_record_resolution(MergeConflictClass::StrategyIntentConflict, &aspects),
        MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::StrategyIntentConflict,
        }
    );
}

#[test]
fn last_writer_wins_rejects_when_causal_order_is_insufficient() {
    let boundary = decision_boundary_for_aspect(
        &crate::merge::data::MergeConflictClassification {
            record: RecordRef::Entity(EntityId::new(PartitionId::main(), 1, 1)),
            class: MergeConflictClass::DivergentVisibleState,
            identity_reason:
                crate::merge::data::IdentityResolutionReason::DeclaredBasisNoVisibleTargetMatch,
            validated_schema_correspondence: false,
            aspect_evidence: Arc::from(Vec::new()),
            strategy_evidence: None,
            relation_evidence: None,
            target_record: None,
            base_record_visible: true,
            source_record_visible: true,
            target_record_visible: true,
            base_visibility_evidence: crate::merge::data::MergeVisibilityEvidence {
                observed_record: RecordRef::Entity(EntityId::new(PartitionId::main(), 1, 1)),
                kind: crate::merge::data::MergeVisibilityEvidenceKind::BaseHistoricalWindow,
                state: crate::merge::data::MergeVisibilityState::Visible,
                embedded_surface_state: None,
                lifecycle: None,
                created_at_version: None,
                retired_at_version: None,
            },
            source_visibility_evidence: crate::merge::data::MergeVisibilityEvidence {
                observed_record: RecordRef::Entity(EntityId::new(PartitionId::main(), 1, 1)),
                kind: crate::merge::data::MergeVisibilityEvidenceKind::SourceEmbeddedSurface,
                state: crate::merge::data::MergeVisibilityState::Visible,
                embedded_surface_state: None,
                lifecycle: None,
                created_at_version: None,
                retired_at_version: None,
            },
            target_visibility_evidence: crate::merge::data::MergeVisibilityEvidence {
                observed_record: RecordRef::Entity(EntityId::new(PartitionId::main(), 1, 1)),
                kind: crate::merge::data::MergeVisibilityEvidenceKind::TargetEmbeddedSurface,
                state: crate::merge::data::MergeVisibilityState::Visible,
                embedded_surface_state: None,
                lifecycle: None,
                created_at_version: None,
                retired_at_version: None,
            },
        },
        crate::merge::data::AspectComparisonState::Divergent,
        Some(&AspectMergePolicyKind::LastWriterWins),
        crate::merge::data::MergeRecordCausalDisposition::Concurrent,
    );

    assert_eq!(
        boundary,
        MergePolicyDecisionBoundary::Reject {
            class: MergePolicyRejectClass::LastWriterWinsCausalConflict,
        }
    );
}
