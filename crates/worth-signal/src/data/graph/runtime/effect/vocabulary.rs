use crate::data::core_profile::StableHashValue;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::output::OutputChange;
use crate::diagnostics::policy::ArtifactRetentionPolicy;
use crate::logic::evaluation::{EvaluationEffect, EvaluationVerdict, SuppressionReason};

pub(super) fn count_changed_partitions(
    changed_regions: &[crate::data::output::ChangedRegion],
    work: &mut crate::logic::evaluation::EvaluationWork<'_>,
) -> Result<u32, crate::data::error::SignalError> {
    work.reserve(
        changed_regions
            .len()
            .checked_mul(std::mem::size_of::<&crate::data::output::PartitionToken>() + 1)
            .filter(|bytes| *bytes <= isize::MAX as usize),
    )?;
    // Borrow partition payloads; counting never needs an owned string copy.
    let mut partitions: Vec<&crate::data::output::PartitionToken> =
        Vec::with_capacity(changed_regions.len());
    for region in changed_regions {
        work.reserve(
            region
                .partition
                .0
                .len()
                .checked_mul(2)
                .and_then(|n| n.checked_add(2))
                .and_then(|cost| cost.checked_mul(partitions.len())),
        )?;
        if !partitions
            .iter()
            .any(|partition| *partition == &region.partition)
        {
            partitions.push(&region.partition);
        }
    }
    u32::try_from(partitions.len()).map_err(|_| {
        crate::data::error::SignalError::invalid_input("changed partition count exceeds u32")
    })
}

pub(super) fn verdict_retains_runtime_artifact(verdict: &EvaluationVerdict) -> bool {
    matches!(
        verdict,
        EvaluationVerdict::Recomputed
            | EvaluationVerdict::Suppressed {
                reason: SuppressionReason::OutputIdentityUnchanged
                    | SuppressionReason::ContinuityTokenUnchanged
                    | SuppressionReason::ComparatorMatch,
            }
    )
}

pub(super) fn verdict_transitions_clean(verdict: &EvaluationVerdict) -> bool {
    matches!(
        verdict,
        EvaluationVerdict::Recomputed | EvaluationVerdict::Suppressed { .. }
    )
}

pub(super) fn verdict_commits_snapshot(verdict: &EvaluationVerdict) -> bool {
    verdict_retains_runtime_artifact(verdict)
}

pub(super) fn normalize_output_change(
    declared: OutputChange,
    output_identity_unchanged: bool,
    has_output_identity: bool,
) -> OutputChange {
    if has_output_identity && output_identity_unchanged {
        OutputChange::Unchanged
    } else {
        declared
    }
}

pub(super) fn trace_identity_hash(
    identity: &crate::data::output::OutputIdentity,
) -> StableHashValue {
    identity.stable_hash()
}

pub(super) fn trace_output_hash(version: crate::data::aspect::AspectVersion) -> StableHashValue {
    let mut hash = 0xcbf29ce484222325_u128;
    for slot in version.slots() {
        hash ^= *slot as u128;
        hash = hash.wrapping_mul(0x100000001b3_u128);
    }
    hash as StableHashValue
}

pub(super) fn runtime_policy_omits_cold_artifacts(graph: &SignalGraph) -> bool {
    let retention = graph.installed_runtime_policy().retention_budget();
    matches!(
        retention.explanation_retention,
        ArtifactRetentionPolicy::Omit
    ) && matches!(
        retention.provenance_retention,
        ArtifactRetentionPolicy::Omit
    )
}

pub(super) fn record_reuse_telemetry(
    telemetry: &mut crate::data::telemetry::RuntimeTelemetry,
    effect: &EvaluationEffect,
) {
    telemetry.evaluation.reuse_eligibility_checks_attempted += 1;
    match effect.operational.reuse_origin {
        crate::data::reuse::ReuseOrigin::FreshCompute => {
            telemetry.evaluation.fresh_compute_count += 1
        }
        crate::data::reuse::ReuseOrigin::OutputSuppressed => {
            telemetry.evaluation.output_suppressed_count += 1
        }
        crate::data::reuse::ReuseOrigin::MemoizedArtifactReuse => {
            telemetry.evaluation.memoized_reuse_count += 1
        }
        crate::data::reuse::ReuseOrigin::SnapshotRestore => {
            telemetry.evaluation.snapshot_restore_reuse_count += 1
        }
        crate::data::reuse::ReuseOrigin::ReconciliationAdoption => {
            telemetry.evaluation.reconciliation_adoption_count += 1
        }
        crate::data::reuse::ReuseOrigin::CrossIdentityPersistentReuse => {
            telemetry.evaluation.cross_identity_reuse_count += 1
        }
        crate::data::reuse::ReuseOrigin::PartialArtifactSplice => {
            telemetry.evaluation.partial_artifact_splice_count += 1
        }
    }
    telemetry.evaluation.reuse_dependency_comparison_breadth +=
        u64::from(effect.operational.meaningful_input_changes);
    if effect.reuse_certification().is_some() {
        telemetry
            .evaluation
            .reuse_cold_certification_materialization_count += 1;
    }
}
