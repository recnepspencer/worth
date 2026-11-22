use worth_store::physical_runtime::{
    PhysicalRecoveryFreshnessPort, StoreRecoveryBindingFreshnessSample, StoreRecoverySamplingBasis,
};
use worth_store_physical_format::{
    PhysicalRecordFormatDeclaration, PhysicalRecoveryProjectionDecodeLimits,
};
use worth_store_recovery_physics::{
    admit_physical_redo_members_with_memory_budget, AdmittedPhysicalRedoMembers,
    PhysicalRedoAdmissionLimits, PhysicalRedoMemberInput, PhysicalRedoPlanCounters,
    PhysicalRedoTargetIdentity, ReconciledOperationFate, ReconciledOperationFates,
    RecoveryOperationEvidenceInput, RecoveryOperationFate, RecoveryPlanCostDenial,
    RecoveryPlanningCounters,
};

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlanningDenial,
};
use crate::orchestration::recovery_budget::RecoveryAllowance;
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};

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

fn memory_block(
    context: PlanningContext,
    counters: RecoveryPlanningCounters,
    denial: PlanningMemoryDenial,
) -> PhysicalRecoveryOutcome {
    match denial {
        PlanningMemoryDenial::RecoveryMemoryBytes { observed } => {
            let limit = RecoveryAllowance::declared(
                &context.limits,
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
            )
            .past(observed);
            context.cost_denial_block(
                counters.with_peak_recovery_bytes(observed),
                RecoveryPlanCostDenial::RecoveryMemoryBytes,
                limit,
            )
        }
        PlanningMemoryDenial::Allocation {
            requested_bytes,
            cause,
        } => context.block_with_planning_attempt_denial(
            PhysicalRecoveryBlockKind::RedoPlanning,
            counters,
            "admitted-planning-basis-allocation",
            None,
            PhysicalRecoveryPlanningDenial::ResolvedStateAllocation {
                requested_bytes,
                cause,
            },
        ),
    }
}

pub(super) fn admit(
    context: PlanningContext,
    mut allowance: PlanningResidentAllowance,
) -> Result<(PlanningContext, AdmittedPlanningBasis), PhysicalRecoveryOutcome> {
    // Selection admitted no checkpoint only over the authenticated absent
    // arm, whose witness keys the generation-zero basis; its policy is the
    // configured declaration.
    let basis = match (
        context.selection.checkpoint(),
        context.coordination.absent_checkpoint(),
        context.authority.durability,
    ) {
        (Some(checkpoint), _, _) => StoreRecoverySamplingBasis::Checkpoint(checkpoint.checkpoint()),
        (None, Some(absent), Some(declaration)) => {
            StoreRecoverySamplingBasis::GenerationZero(absent, declaration)
        }
        (None, _, _) => {
            return Err(context.block(
                PhysicalRecoveryBlockKind::Checkpoint,
                "generation-zero-requires-durability-declaration",
                None,
            ))
        }
    };
    let sampling_budget = allowance.remaining();
    let sample = match PhysicalRecoveryFreshnessPort::sample_binding_with_budget(
        context.coordination.owner(),
        &context.authority.media,
        basis,
        context
            .integrity
            .admitted_wal()
            .recoverable_frame_view(context.selection.wal_tail()),
        context.limits.operation_bindings,
        context.limits.redo_bytes,
        sampling_budget,
        sampling_budget,
    ) {
        Ok(sample) => sample,
        Err(failure) => {
            let denial = failure.denial();
            let limit = sample_limit(&failure, &context.limits);
            let planning_counters =
                counters::failed_sample(failure.freshness_retained(), failure.freshness_expired());
            return Err(context.block_with_planning_attempt_denial(
                PhysicalRecoveryBlockKind::BindingFreshness,
                planning_counters,
                "binding-freshness-sample",
                limit,
                match failure.allocation_denial() {
                    Some(cause) => {
                        PhysicalRecoveryPlanningDenial::BindingSamplingAllocation(cause.clone())
                    }
                    None => PhysicalRecoveryPlanningDenial::BindingFreshness(denial),
                },
            ));
        }
    };
    let sample_bytes = sample
        .owned_heap_bytes()
        .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX });
    if let Err(denial) = sample_bytes.and_then(|bytes| allowance.retain(bytes)) {
        let planning_counters = counters::after_sample(&sample);
        return Err(memory_block(context, planning_counters, denial));
    }
    let fates = match admit_reconciled_fates(&sample, context.limits.operation_bindings, &mut allowance) {
        Ok(fates) => fates,
        Err(ReconcileAdmissionError::Memory(denial)) => {
            let planning_counters = counters::after_sample(&sample);
            return Err(memory_block(context, planning_counters, denial));
        }
        Err(ReconcileAdmissionError::Reconciliation(denial)) => {
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
    let redo_copy_bytes = sample
        .wal_members()
        .iter()
        .try_fold(0_u64, |total, member| {
            total.checked_add(member.canonical_redo().len() as u64)
        })
        .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX });
    let redo_inputs_live =
        PlanningResidentAllowance::slot_bytes::<PhysicalRedoMemberInput>(sample.wal_members().len())
            .and_then(|vec_bytes| {
                vec_bytes
                    .checked_add(redo_copy_bytes?)
                    .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX })
            });
    let redo_inputs_transient = redo_inputs_live.clone().and_then(|live| {
        let index = PlanningResidentAllowance::slot_bytes::<([u8; 32], RecoveryOperationFate)>(
            fates.operations().len(),
        )?;
        live.checked_add(index)
            .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX })
    });
    if let Err(denial) = redo_inputs_transient.and_then(|bytes| allowance.transient(bytes)) {
        let planning_counters =
            counters::after_fates(&sample, &fates, PhysicalRedoPlanCounters::default(), 0, 0);
        return Err(memory_block(context, planning_counters, denial));
    }
    let redo_inputs_live = match redo_inputs_live {
        Ok(bytes) => {
            if let Err(denial) = allowance.retain(bytes) {
                let planning_counters = counters::after_fates(
                    &sample,
                    &fates,
                    PhysicalRedoPlanCounters::default(),
                    0,
                    0,
                );
                return Err(memory_block(context, planning_counters, denial));
            }
            bytes
        }
        Err(denial) => {
            let planning_counters =
                counters::after_fates(&sample, &fates, PhysicalRedoPlanCounters::default(), 0, 0);
            return Err(memory_block(context, planning_counters, denial));
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
    let redo_budget = allowance.remaining();
    let redo = match admit_physical_redo_members_with_memory_budget(
        redo_inputs,
        context.authority.media.store_identity(),
        format,
        PhysicalRedoAdmissionLimits {
            recovery_memory_bytes: redo_budget,
            targets: context.limits.redo_targets,
            distinct_targets: context.limits.distinct_pages_and_extents,
            projection: PhysicalRecoveryProjectionDecodeLimits {
                frames: context.limits.redo_targets,
                record_identities: context.limits.redo_targets,
                placements: remaining_manifest_entries,
                segment_updates: remaining_manifest_entries,
                manifests: redo_bytes,
                total_entries: remaining_manifest_entries
                    .saturating_mul(2)
                    .saturating_add(redo_bytes),
                inline_allocations: remaining_manifest_entries,
            },
        },
        redo_budget,
    ) {
        Ok(redo) => redo,
        Err(denial) => {
            let planning_counters =
                counters::after_fates(&sample, &fates, PhysicalRedoPlanCounters::default(), 0, 0);
            return Err(context.redo_denial_block(planning_counters, denial));
        }
    };
    // `admit_physical_redo_members` consumed `redo_inputs`; replace its debit
    // with the retained admitted redo before admitting the target roster.
    allowance.release(redo_inputs_live);
    let redo_live_bytes = redo
        .owned_heap_bytes()
        .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX });
    if let Err(denial) = redo_live_bytes.and_then(|bytes| allowance.retain(bytes)) {
        let planning_counters =
            counters::after_fates(&sample, &fates, PhysicalRedoPlanCounters::default(), 0, 0);
        return Err(memory_block(context, planning_counters, denial));
    }
    let targets = match admit_target_identities(&redo, &mut allowance) {
        Ok(targets) => targets,
        Err(denial) => {
            let planning_counters =
                counters::after_fates(&sample, &fates, PhysicalRedoPlanCounters::default(), 0, 0);
            return Err(memory_block(context, planning_counters, denial));
        }
    };
    let distinct_targets = redo.distinct_target_count();
    let distinct = RecoveryAllowance::declared(
        &context.limits,
        PhysicalRecoveryLimitDimension::DistinctPagesAndExtents,
    );
    if let Some(limit) = distinct.past(distinct_targets) {
        let planning_counters =
            counters::after_fates(&sample, &fates, PhysicalRedoPlanCounters::default(), 0, 0);
        return Err(context.redo_block(planning_counters, Some(limit)));
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

#[derive(Debug)]
pub(super) enum ReconcileAdmissionError {
    Memory(PlanningMemoryDenial),
    Reconciliation(worth_store_recovery_physics::OperationReconciliationDenial),
}

pub(super) fn admit_reconciled_fates(
    sample: &StoreRecoveryBindingFreshnessSample,
    maximum_bindings: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<ReconciledOperationFates, ReconcileAdmissionError> {
    let inputs = PlanningResidentAllowance::slot_bytes::<RecoveryOperationEvidenceInput>(
        sample.operations().len(),
    )
    .map_err(ReconcileAdmissionError::Memory)?;
    let output = PlanningResidentAllowance::slot_bytes::<ReconciledOperationFate>(
        sample.operations().len(),
    )
    .map_err(ReconcileAdmissionError::Memory)?;
    let transient = inputs
        .checked_add(output)
        .ok_or(ReconcileAdmissionError::Memory(
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        ))?;
    allowance
        .transient(transient)
        .map_err(ReconcileAdmissionError::Memory)?;
    let fates = operation_join::reconcile_sample(sample, maximum_bindings)
        .map_err(ReconcileAdmissionError::Reconciliation)?;
    let retained = fates
        .owned_heap_bytes()
        .ok_or(ReconcileAdmissionError::Memory(
            PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
        ))?;
    allowance
        .retain(retained)
        .map_err(ReconcileAdmissionError::Memory)?;
    Ok(fates)
}

pub(super) fn admit_target_identities(
    redo: &AdmittedPhysicalRedoMembers,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Box<[PhysicalRedoTargetIdentity]>, PlanningMemoryDenial> {
    let targets_bytes =
        PlanningResidentAllowance::slot_bytes::<PhysicalRedoTargetIdentity>(
            redo.target_identity_count(),
        )?;
    allowance.retain(targets_bytes)?;
    Ok(redo.target_identities())
}

#[cfg(test)]
#[path = "admitted_basis_tests.rs"]
mod tests;
