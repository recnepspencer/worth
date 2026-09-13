use crate::backup_verification::verification_support::{
    backup_truth_evidence, map_owner_binding_denial, verification_identity,
};
use crate::inspection::OwnerDecodedArtifactBinding;
use crate::truth_composition::RecoveryCandidateObservation;
use crate::{
    truth_composition::compose_operational_truth_with_owner_candidates, OfflineInspectionBudget,
    OperationalTruthCompositionBudget, OperationalTruthReport, StructurallyWalkedMedia,
};
use worth_store_physical_format::MaterializedBackupBundle;

use super::{BackupStructuralVerificationDenial, BackupVerificationAllocationPhase};

pub(super) struct CompletedBackupTruth {
    pub(super) verification_identity: [u8; 32],
    pub(super) operational_truth: OperationalTruthReport,
    pub(super) peak_owned_allocation_bytes: u64,
}

pub(super) struct BackupTruthCompletion<'a> {
    pub(super) current: &'a MaterializedBackupBundle,
    pub(super) walked: StructurallyWalkedMedia,
    pub(super) defect_owned_allocation_bytes: u64,
    pub(super) owner_bindings: Vec<OwnerDecodedArtifactBinding>,
    pub(super) admitted_auxiliary_components:
        &'a [worth_store_physical_backend::OfflineMediaClosureEntry],
    pub(super) recovery_candidates: Vec<RecoveryCandidateObservation>,
    pub(super) inspection_budget: OfflineInspectionBudget,
    pub(super) cancellation: &'a crate::OfflineInspectionCancellation,
    pub(super) started_at: std::time::Instant,
}

pub(super) fn complete_backup_truth(
    completion: BackupTruthCompletion<'_>,
) -> Result<CompletedBackupTruth, BackupStructuralVerificationDenial> {
    let BackupTruthCompletion {
        current,
        mut walked,
        defect_owned_allocation_bytes,
        owner_bindings,
        admitted_auxiliary_components,
        recovery_candidates,
        inspection_budget,
        cancellation,
        started_at,
    } = completion;
    let verification_identity = verification_identity(current, &walked);
    if !walked.remove_auxiliary_components(admitted_auxiliary_components) {
        return Err(BackupStructuralVerificationDenial::OwnerBindingMissingSource);
    }
    walked
        .bind_owner_observations(owner_bindings)
        .map_err(map_owner_binding_denial)?;
    let outside_owned_allocation_bytes = current
        .manifest_read_observation()
        .owned_allocation_bytes()
        .checked_add(defect_owned_allocation_bytes)
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    let composition_limit = inspection_budget
        .maximum_owned_allocation_bytes()
        .checked_sub(outside_owned_allocation_bytes)
        .ok_or(
            BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
                phase: BackupVerificationAllocationPhase::TruthComposition,
                admitted: outside_owned_allocation_bytes,
                limit: inspection_budget.maximum_owned_allocation_bytes(),
            },
        )?;
    let truth_evidence = backup_truth_evidence(current, composition_limit)
        .map_err(|denial| rebase_owned_allocation_denial(denial, outside_owned_allocation_bytes))?;
    let zero_composition_admitted = outside_owned_allocation_bytes
        .checked_add(1)
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    let composition_budget = OperationalTruthCompositionBudget::bounded(composition_limit).ok_or(
        BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
            phase: BackupVerificationAllocationPhase::TruthComposition,
            admitted: zero_composition_admitted,
            limit: inspection_budget.maximum_owned_allocation_bytes(),
        },
    )?;
    let operational_truth = compose_operational_truth_with_owner_candidates(
        walked,
        &truth_evidence,
        recovery_candidates,
        composition_budget,
        &mut || {
            crate::inspection::reject_inspection_interruption(
                inspection_budget,
                cancellation,
                started_at,
            )
        },
    )
    .map_err(|denial| match denial {
        crate::OperationalTruthCompositionDenial::OwnedAllocationBudgetExceeded {
            admitted,
            limit,
        } => rebase_budget_denial(
            BackupVerificationAllocationPhase::TruthComposition,
            admitted,
            limit,
            outside_owned_allocation_bytes,
        ),
        crate::OperationalTruthCompositionDenial::Interrupted(denial) => {
            BackupStructuralVerificationDenial::Inspection(denial)
        }
        other => BackupStructuralVerificationDenial::TruthComposition(other),
    })?;
    if operational_truth.regions().iter().any(|region| {
        matches!(
            region,
            crate::OperationalTruthRegion::OverlapConflict { .. }
        )
    }) {
        return Err(BackupStructuralVerificationDenial::PhysicalOwnershipOverlap);
    }
    let peak_owned_allocation_bytes = outside_owned_allocation_bytes
        .checked_add(operational_truth.peak_owned_allocation_bytes())
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    Ok(CompletedBackupTruth {
        verification_identity,
        operational_truth,
        peak_owned_allocation_bytes,
    })
}

fn rebase_owned_allocation_denial(
    denial: BackupStructuralVerificationDenial,
    outside_owned_allocation_bytes: u64,
) -> BackupStructuralVerificationDenial {
    match denial {
        BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
            phase,
            admitted,
            limit,
        } => rebase_budget_denial(phase, admitted, limit, outside_owned_allocation_bytes),
        other => other,
    }
}

fn rebase_budget_denial(
    phase: BackupVerificationAllocationPhase,
    admitted: u64,
    limit: u64,
    outside: u64,
) -> BackupStructuralVerificationDenial {
    match (outside.checked_add(admitted), outside.checked_add(limit)) {
        (Some(admitted), Some(limit)) => {
            BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
                phase,
                admitted,
                limit,
            }
        }
        _ => BackupStructuralVerificationDenial::VerificationCounterOverflow,
    }
}
