use std::collections::BTreeSet;

use worth_store::physical_runtime::{
    PhysicalRecoveryFreshnessPort, StoreRecoveryBindingFreshnessSample,
};
use worth_store_physical_format::{
    PhysicalRecordFormatDeclaration, PhysicalRecoveryProjectionDecodeLimits,
};
use worth_store_recovery_physics::{
    admit_physical_redo_members, AdmittedPhysicalRedoMembers, PhysicalRedoAdmissionLimits,
    PhysicalRedoPlanCounters, PhysicalRedoTargetIdentity, ReconciledOperationFates,
};

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure,
    PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
};

use super::context::PlanningContext;
use super::counters;
use super::denial::sample_limit;
use super::operation_join;

pub(super) struct AdmittedPlanningBasis {
    pub(super) sample: StoreRecoveryBindingFreshnessSample,
    pub(super) fates: ReconciledOperationFates,
    pub(super) redo: AdmittedPhysicalRedoMembers,
    pub(super) targets: Box<[PhysicalRedoTargetIdentity]>,
    pub(super) redo_bytes: u64,
    pub(super) distinct_targets: u64,
    pub(super) remaining_manifest_entries: u64,
    pub(super) format: PhysicalRecordFormatDeclaration,
}

pub(super) fn admit(
    context: PlanningContext,
) -> Result<(PlanningContext, AdmittedPlanningBasis), PhysicalRecoveryOutcome> {
    let Some(checkpoint) = context.selection.checkpoint() else {
        return Err(context.block(
            PhysicalRecoveryBlockKind::Checkpoint,
            "phase-4-requires-selected-checkpoint",
            None,
        ));
    };
    let sample = match PhysicalRecoveryFreshnessPort::sample_binding(
        context.coordination.owner(),
        &context.authority.media,
        checkpoint.checkpoint(),
        context
            .integrity
            .admitted_wal()
            .selected_frames(context.selection.wal_tail()),
        context.limits.operation_bindings,
        context.limits.redo_bytes,
    ) {
        Ok(sample) => sample,
        Err(failure) => {
            let denial = failure.denial();
            let limit = sample_limit(
                failure,
                context.limits.operation_bindings,
                context.limits.redo_bytes,
            );
            let planning_counters =
                counters::failed_sample(failure.freshness_retained(), failure.freshness_expired());
            return Err(context.block_with_planning_attempt_denial(
                PhysicalRecoveryBlockKind::BindingFreshness,
                planning_counters,
                "binding-freshness-sample",
                limit,
                PhysicalRecoveryPlanningDenial::BindingFreshness(denial),
            ));
        }
    };
    let fates = match operation_join::reconcile_sample(&sample, context.limits.operation_bindings) {
        Ok(fates) => fates,
        Err(denial) => {
            let planning_counters = counters::after_sample(&sample);
            return Err(context.block_with_planning_attempt_denial(
                PhysicalRecoveryBlockKind::OperationReconciliation,
                planning_counters,
                "operation-fate-reconciliation",
                None,
                PhysicalRecoveryPlanningDenial::OperationReconciliation(denial),
            ));
        }
    };
    let redo_inputs = match operation_join::redo_inputs(&sample, &fates) {
        Ok(inputs) => inputs,
        Err(denial) => {
            let planning_counters =
                counters::after_fates(&sample, &fates, PhysicalRedoPlanCounters::default(), 0, 0);
            return Err(context.block_with_planning_attempt_denial(
                PhysicalRecoveryBlockKind::OperationReconciliation,
                planning_counters,
                "wal-operation-evidence-join",
                None,
                PhysicalRecoveryPlanningDenial::OperationReconciliation(denial),
            ));
        }
    };
    let redo_bytes = redo_inputs
        .iter()
        .map(|member| member.canonical_redo().len() as u64)
        .sum();
    let remaining_manifest_entries = context
        .limits
        .manifest_entries
        .saturating_sub(context.counters.manifest_entries);
    let format = context.selection.root().selected().selector().format();
    let redo = match admit_physical_redo_members(
        redo_inputs,
        context.authority.media.store_identity(),
        format,
        PhysicalRedoAdmissionLimits {
            recovery_memory_bytes: context.limits.recovery_memory_bytes,
            targets: context.limits.redo_targets,
            distinct_targets: context.limits.distinct_pages_and_extents,
            projection: PhysicalRecoveryProjectionDecodeLimits {
                frames: context.limits.redo_targets,
                record_identities: context.limits.redo_targets,
                placements: remaining_manifest_entries,
                segment_updates: remaining_manifest_entries,
                manifests: remaining_manifest_entries,
                total_entries: remaining_manifest_entries,
                inline_allocations: remaining_manifest_entries,
            },
        },
    ) {
        Ok(redo) => redo,
        Err(denial) => {
            let planning_counters =
                counters::after_fates(&sample, &fates, PhysicalRedoPlanCounters::default(), 0, 0);
            let limit = match denial {
                worth_store_recovery_physics::PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
                    observed,
                    admitted,
                } => Some(PhysicalRecoveryLimitFailure {
                    dimension: PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
                    observed,
                    admitted,
                }),
                _ => None,
            };
            return Err(context.redo_denial_block(planning_counters, limit, denial));
        }
    };
    let targets = redo.target_identities();
    let distinct_targets = targets.iter().copied().collect::<BTreeSet<_>>().len() as u64;
    if distinct_targets > context.limits.distinct_pages_and_extents {
        let admitted = context.limits.distinct_pages_and_extents;
        let planning_counters =
            counters::after_fates(&sample, &fates, PhysicalRedoPlanCounters::default(), 0, 0);
        return Err(context.redo_block(
            planning_counters,
            Some(PhysicalRecoveryLimitFailure {
                dimension: PhysicalRecoveryLimitDimension::DistinctPagesAndExtents,
                observed: distinct_targets,
                admitted,
            }),
        ));
    }
    Ok((
        context,
        AdmittedPlanningBasis {
            sample,
            fates,
            redo,
            targets,
            redo_bytes,
            distinct_targets,
            remaining_manifest_entries,
            format,
        },
    ))
}
