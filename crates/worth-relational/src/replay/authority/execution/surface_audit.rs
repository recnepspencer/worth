use crate::capabilities::{HistorySource, ReplayRead};
use crate::history::data::HistoryDriftClass;
use crate::replay::data::{
    CanonicalCommitEnvelope, RelationalReplayRequest, ReplayMismatch, ReplayMismatchClass,
    ReplayObservableSurface, ReplayVerificationLayer, ReplayVerificationPlan,
};
use crate::runtime::RelationalRuntime;

use super::super::super::derived_index_surface::{
    surface_basis_for_derived_index_artifacts, DERIVED_INDEX_SURFACE,
};
use super::super::lineage_authority::published_lineage_artifacts_match;
use super::super::surface_comparison::{
    compare_descriptor_surface, compare_replay_surface, surface_basis_for_branch_head,
    surface_basis_for_diagnostics, surface_basis_for_history, surface_basis_for_patch,
    surface_basis_for_published_lineage, surface_basis_for_snapshot, surface_basis_for_strategy,
};
use super::super::{SelectedPublishedLineageAuthority, ValidatedReplayContinuityEnvelope};

pub(super) fn compare_replay_surfaces(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    verification_plan: &ReplayVerificationPlan,
    mismatches: &mut Vec<ReplayMismatch>,
    compared_surfaces: &[ReplayObservableSurface],
    envelope: &CanonicalCommitEnvelope,
    replayed_envelope: &CanonicalCommitEnvelope,
    validated_envelope: &ValidatedReplayContinuityEnvelope<'_>,
    validated_replayed_envelope: &ValidatedReplayContinuityEnvelope<'_>,
    replay_runtime: &RelationalRuntime,
    request: &RelationalReplayRequest,
    selected_lineage_authority: Option<&SelectedPublishedLineageAuthority>,
) {
    runtime
        .performance_access()
        .count_merge_history_parent_comparisons(
            envelope
                .commit
                .ordered_parents()
                .len()
                .max(replayed_envelope.commit.ordered_parents().len()),
        );
    compare_replay_surface(
        runtime,
        verification_plan,
        mismatches,
        ReplayObservableSurface::Patch,
        ReplayMismatchClass::PatchDrift,
        surface_basis_for_patch(envelope),
        surface_basis_for_patch(replayed_envelope),
        "canonical patch artifact differed",
        || envelope.patch.canonicalized() == replayed_envelope.patch.canonicalized(),
        || format!("{:?}", envelope.patch),
        || format!("{:?}", replayed_envelope.patch),
    );
    compare_replay_surface(
        runtime,
        verification_plan,
        mismatches,
        ReplayObservableSurface::Diagnostics,
        ReplayMismatchClass::DiagnosticsDrift,
        surface_basis_for_diagnostics(envelope),
        surface_basis_for_diagnostics(replayed_envelope),
        "diagnostics summary differed",
        || envelope.diagnostics_summary == replayed_envelope.diagnostics_summary,
        || format!("{:?}", envelope.diagnostics_summary),
        || format!("{:?}", replayed_envelope.diagnostics_summary),
    );
    compare_replay_surface(
        runtime,
        verification_plan,
        mismatches,
        ReplayObservableSurface::History,
        ReplayMismatchClass::HistoryDrift,
        surface_basis_for_history(envelope),
        surface_basis_for_history(replayed_envelope),
        "authoritative ordered parent history differed",
        || {
            envelope.commit.ordered_parents() == replayed_envelope.commit.ordered_parents()
                && envelope.merge_parent_branches == replayed_envelope.merge_parent_branches
                && envelope.merge_base_commits == replayed_envelope.merge_base_commits
        },
        || {
            format!(
                "{:?}|{:?}|{:?}",
                envelope.commit.ordered_parents().as_slice(),
                envelope.merge_parent_branches,
                envelope.merge_base_commits
            )
        },
        || {
            format!(
                "{:?}|{:?}|{:?}",
                replayed_envelope.commit.ordered_parents().as_slice(),
                replayed_envelope.merge_parent_branches,
                replayed_envelope.merge_base_commits
            )
        },
    );
    if compared_surfaces.contains(&ReplayObservableSurface::Strategy) {
        compare_replay_surface(
            runtime,
            verification_plan,
            mismatches,
            ReplayObservableSurface::Strategy,
            ReplayMismatchClass::StrategyArtifactDrift,
            surface_basis_for_strategy(envelope.strategy_artifacts.as_ref()),
            surface_basis_for_strategy(replayed_envelope.strategy_artifacts.as_ref()),
            "strategy replay descriptor differed",
            || envelope.strategy_artifacts == replayed_envelope.strategy_artifacts,
            || format!("{:?}", envelope.strategy_artifacts),
            || format!("{:?}", replayed_envelope.strategy_artifacts),
        );
    }
    if replayed_envelope.descriptor_semantics_version != envelope.descriptor_semantics_version {
        runtime
            .performance_access()
            .count_descriptor_version_mismatch();
        runtime
            .performance_access()
            .count_replay_verification_layer(ReplayVerificationLayer::DigestParity);
        mismatches.push(ReplayMismatch {
            class: ReplayMismatchClass::DescriptorVersionDrift,
            history_drift_class: Some(HistoryDriftClass::ReplayAuthorityDrift),
            surface: ReplayObservableSurface::History,
            verification_layer: ReplayVerificationLayer::DigestParity,
            detail: "descriptor semantics version differed".to_string(),
            expected: Some(format!("{:?}", envelope.descriptor_semantics_version)),
            observed: Some(format!(
                "{:?}",
                replayed_envelope.descriptor_semantics_version
            )),
        });
    }
    compare_descriptor_surface(
        runtime,
        verification_plan,
        mismatches,
        validated_envelope.transition_basis.clone(),
        validated_replayed_envelope.transition_basis.clone(),
        ReplayMismatchClass::SchemaTransitionDrift,
        "schema transition artifact differed",
        || envelope.schema_transition == replayed_envelope.schema_transition,
        || format!("{:?}", envelope.schema_transition),
        || format!("{:?}", replayed_envelope.schema_transition),
    );
    compare_descriptor_surface(
        runtime,
        verification_plan,
        mismatches,
        validated_envelope.continuation_basis.clone(),
        validated_replayed_envelope.continuation_basis.clone(),
        ReplayMismatchClass::SchemaContinuationDescriptorDrift,
        "schema continuation descriptor differed",
        || {
            envelope.schema_continuation_descriptor
                == replayed_envelope.schema_continuation_descriptor
        },
        || format!("{:?}", envelope.schema_continuation_descriptor),
        || format!("{:?}", replayed_envelope.schema_continuation_descriptor),
    );
    compare_descriptor_surface(
        runtime,
        verification_plan,
        mismatches,
        validated_envelope.reconciliation_basis.clone(),
        validated_replayed_envelope.reconciliation_basis.clone(),
        ReplayMismatchClass::SchemaReconciliationDescriptorDrift,
        "schema reconciliation descriptor differed",
        || {
            envelope.schema_reconciliation_descriptor
                == replayed_envelope.schema_reconciliation_descriptor
        },
        || format!("{:?}", envelope.schema_reconciliation_descriptor),
        || format!("{:?}", replayed_envelope.schema_reconciliation_descriptor),
    );
    compare_descriptor_surface(
        runtime,
        verification_plan,
        mismatches,
        validated_envelope.lineage_basis.clone(),
        validated_replayed_envelope.lineage_basis.clone(),
        ReplayMismatchClass::SchemaLineageDrift,
        "schema lineage artifact differed",
        || {
            envelope
                .schema_reconciliation_descriptor
                .as_ref()
                .map(|descriptor| &descriptor.resulting_lineage)
                == replayed_envelope
                    .schema_reconciliation_descriptor
                    .as_ref()
                    .map(|descriptor| &descriptor.resulting_lineage)
        },
        || {
            format!(
                "{:?}",
                envelope
                    .schema_reconciliation_descriptor
                    .as_ref()
                    .map(|descriptor| &descriptor.resulting_lineage)
            )
        },
        || {
            format!(
                "{:?}",
                replayed_envelope
                    .schema_reconciliation_descriptor
                    .as_ref()
                    .map(|descriptor| &descriptor.resulting_lineage)
            )
        },
    );
    if compared_surfaces.contains(&ReplayObservableSurface::Snapshot) {
        let original_surface =
            runtime.replay_snapshot_surface_at_version(execution, envelope.commit.version_id);
        let replayed_surface = replay_runtime
            .replay_snapshot_surface_at_version(execution, replayed_envelope.commit.version_id);
        compare_replay_surface(
            runtime,
            verification_plan,
            mismatches,
            ReplayObservableSurface::Snapshot,
            ReplayMismatchClass::SnapshotDrift,
            surface_basis_for_snapshot(&original_surface),
            surface_basis_for_snapshot(&replayed_surface),
            "snapshot-visible state differed",
            || original_surface == replayed_surface,
            || format!("{:?}", original_surface),
            || format!("{:?}", replayed_surface),
        );
    }
    let expected_branch_head = Some(envelope.commit.clone());
    compare_replay_surface(
        runtime,
        verification_plan,
        mismatches,
        ReplayObservableSurface::BranchHead,
        ReplayMismatchClass::BranchHeadDrift,
        surface_basis_for_branch_head(expected_branch_head.as_ref()),
        surface_basis_for_branch_head(replay_runtime.branch_head_ref(&request.branch_id).as_ref()),
        "branch head movement differed",
        || replay_runtime.branch_head_ref(&request.branch_id) == expected_branch_head,
        || format!("{:?}", expected_branch_head),
        || format!("{:?}", replay_runtime.branch_head_ref(&request.branch_id)),
    );
    if compared_surfaces.contains(&ReplayObservableSurface::Lineage) {
        if let Some(original_lineage) = selected_lineage_authority {
            let replayed_lineage = replayed_envelope.published_lineage();
            compare_replay_surface(
                runtime,
                verification_plan,
                mismatches,
                ReplayObservableSurface::Lineage,
                ReplayMismatchClass::LineageDrift,
                surface_basis_for_published_lineage(&original_lineage.artifact),
                surface_basis_for_published_lineage(replayed_lineage),
                "lineage artifacts differed",
                || published_lineage_artifacts_match(&original_lineage.artifact, replayed_lineage),
                || format!("{:?}", original_lineage.artifact),
                || format!("{:?}", replayed_lineage),
            );
        }
    }
    if compared_surfaces.contains(&DERIVED_INDEX_SURFACE) {
        let original_derived_index_artifacts = crate::indexes::data::DerivedIndexArtifacts::new(
            runtime.index_generations_at_version(envelope.commit.version_id),
        );
        let replayed_derived_index_artifacts = crate::indexes::data::DerivedIndexArtifacts::new(
            replay_runtime.index_generations_at_version(envelope.commit.version_id),
        );
        compare_replay_surface(
            runtime,
            verification_plan,
            mismatches,
            DERIVED_INDEX_SURFACE,
            ReplayMismatchClass::DerivedIndexDrift,
            surface_basis_for_derived_index_artifacts(&original_derived_index_artifacts),
            surface_basis_for_derived_index_artifacts(&replayed_derived_index_artifacts),
            "derived index artifacts differed",
            || replayed_derived_index_artifacts == original_derived_index_artifacts,
            || format!("{:?}", original_derived_index_artifacts),
            || format!("{:?}", replayed_derived_index_artifacts),
        );
    }
}
