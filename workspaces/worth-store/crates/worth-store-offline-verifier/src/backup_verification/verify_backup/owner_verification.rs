use crate::backup_verification::owner_media_read::OwnerMediaReadSession;
use crate::backup_verification::owner_semantic_verification::{
    verify_owner_semantics, OwnerSemanticVerificationCounters, OwnerSemanticVerificationDenial,
};
use crate::backup_verification::verification_owned_memory::defect_owned_allocation_bytes;
use crate::backup_verification::{BackupVerificationDefect, BackupVerificationReadAccounting};
use crate::inspection::OwnerDecodedArtifactBinding;
use crate::truth_composition::RecoveryCandidateObservation;
use crate::{OfflineInspectionBudget, StructurallyWalkedMedia};
use worth_store_physical_format::MaterializedBackupBundle;

use super::structural_comparison::StructuralBackupComparison;
use super::{BackupStructuralVerificationDenial, BackupVerificationAllocationPhase};

pub(super) struct OwnerVerifiedBackupStructure {
    pub(super) defects: Vec<BackupVerificationDefect>,
    pub(super) counters: OwnerSemanticVerificationCounters,
    pub(super) recovery_candidates: Vec<RecoveryCandidateObservation>,
    pub(super) owner_bindings: Vec<OwnerDecodedArtifactBinding>,
    pub(super) admitted_read_bytes: u64,
    pub(super) observed_read_bytes: u64,
    pub(super) read_accounting: BackupVerificationReadAccounting,
    pub(super) inspected_files: u64,
    pub(super) peak_buffer_bytes: u64,
    pub(super) peak_owned_allocation_bytes: u64,
}

pub(super) fn verify_backup_owner_semantics(
    current: &MaterializedBackupBundle,
    walked: &StructurallyWalkedMedia,
    inspection_budget: OfflineInspectionBudget,
    admitted_read_bytes: u64,
    structural: StructuralBackupComparison,
    cancellation: crate::OfflineInspectionCancellation,
    started_at: std::time::Instant,
) -> Result<OwnerVerifiedBackupStructure, BackupStructuralVerificationDenial> {
    let StructuralBackupComparison {
        mut defects,
        base_retained_owned_allocation_bytes,
        retained_owned_allocation_bytes,
        peak_owned_allocation_bytes: structural_peak_owned_allocation_bytes,
    } = structural;
    let owner_owned_allocation_limit = inspection_budget
        .maximum_owned_allocation_bytes()
        .checked_sub(retained_owned_allocation_bytes)
        .ok_or(
            BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
                phase: BackupVerificationAllocationPhase::StructuralComparison,
                admitted: retained_owned_allocation_bytes,
                limit: inspection_budget.maximum_owned_allocation_bytes(),
            },
        )?;
    let minimum_owner_media = retained_owned_allocation_bytes
        .checked_add(inspection_budget.max_buffer_bytes() as u64)
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    let owner_media_budget = inspection_budget
        .with_maximum_owned_allocation_bytes(owner_owned_allocation_limit)
        .ok_or(
            BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
                phase: BackupVerificationAllocationPhase::OwnerSemanticVerification,
                admitted: minimum_owner_media,
                limit: inspection_budget.maximum_owned_allocation_bytes(),
            },
        )?;
    let mut owner_media = OwnerMediaReadSession::open(
        walked.files().iter().map(|file| file.path().to_path_buf()),
        walked.consistency_basis().clone(),
        owner_media_budget,
        cancellation,
        started_at,
    )
    .map_err(|denial| {
        BackupStructuralVerificationDenial::Acquisition(
            crate::OfflineMediaAcquisitionDenial::Media(denial),
        )
    })?;
    let owner_media_open_peak = retained_owned_allocation_bytes
        .checked_add(owner_media.peak_owned_allocation_bytes())
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    if owner_media_open_peak > inspection_budget.maximum_owned_allocation_bytes() {
        return Err(
            BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
                phase: BackupVerificationAllocationPhase::OwnerSemanticVerification,
                admitted: owner_media_open_peak,
                limit: inspection_budget.maximum_owned_allocation_bytes(),
            },
        );
    }
    let owner_media_resident = retained_owned_allocation_bytes
        .checked_add(owner_media.resident_owned_allocation_bytes())
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    let owner_result_allocation_limit = owner_owned_allocation_limit
        .checked_sub(owner_media.resident_owned_allocation_bytes())
        .ok_or(
            BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
                phase: BackupVerificationAllocationPhase::OwnerSemanticVerification,
                admitted: owner_media_resident,
                limit: inspection_budget.maximum_owned_allocation_bytes(),
            },
        )?;
    let owner = verify_owner_semantics(
        current.root(),
        current.manifest(),
        inspection_budget.max_buffer_bytes(),
        owner_result_allocation_limit,
        &mut defects,
        &mut owner_media,
    )
    .map_err(|denial| match denial {
        OwnerSemanticVerificationDenial::Resource(denial) => owner_resource_denial(
            owner_media_resident,
            denial.required_bytes,
            denial.limit_bytes,
        ),
        OwnerSemanticVerificationDenial::AllocationFailed => {
            BackupStructuralVerificationDenial::VerificationAllocationFailed
        }
        OwnerSemanticVerificationDenial::Media(denial) => {
            BackupStructuralVerificationDenial::Acquisition(
                crate::OfflineMediaAcquisitionDenial::Media(denial),
            )
        }
        OwnerSemanticVerificationDenial::Inspection(denial) => {
            BackupStructuralVerificationDenial::Inspection(denial)
        }
    })?;
    owner_media
        .revalidate_consistency()
        .map_err(crate::OfflineInspectionDenial::Media)
        .map_err(BackupStructuralVerificationDenial::Inspection)?;
    let retained_after_owner = base_retained_owned_allocation_bytes
        .checked_add(
            defect_owned_allocation_bytes(&defects)
                .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?,
        )
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    let owner_peak_with_retained = retained_after_owner
        .checked_add(owner_media.resident_owned_allocation_bytes())
        .and_then(|bytes| bytes.checked_add(owner.peak_owned_allocation_bytes))
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    if owner_peak_with_retained > inspection_budget.maximum_owned_allocation_bytes() {
        return Err(
            BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
                phase: BackupVerificationAllocationPhase::OwnerSemanticVerification,
                admitted: owner_peak_with_retained,
                limit: inspection_budget.maximum_owned_allocation_bytes(),
            },
        );
    }
    let inspected_files = walked
        .counters()
        .file_touches()
        .checked_add(owner.counters.artifacts_attempted())
        .and_then(|count| count.checked_add(1))
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    let manifest_read = current.manifest_read_observation();
    let observed_read_bytes = walked
        .counters()
        .bytes_read()
        .checked_add(manifest_read.encoded_bytes())
        .and_then(|bytes| bytes.checked_add(owner.counters.bytes_read()))
        .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?;
    let read_accounting = BackupVerificationReadAccounting::Complete;
    Ok(OwnerVerifiedBackupStructure {
        defects,
        counters: owner.counters,
        recovery_candidates: owner.recovery_candidates,
        owner_bindings: owner.owner_bindings,
        admitted_read_bytes,
        observed_read_bytes,
        read_accounting,
        inspected_files,
        peak_buffer_bytes: walked
            .counters()
            .peak_buffer_bytes()
            .max(owner.counters.peak_buffer_bytes())
            .max(manifest_read.read_buffer_bytes()),
        peak_owned_allocation_bytes: manifest_read
            .owned_allocation_bytes()
            .checked_add(walked.counters().peak_owned_allocation_bytes())
            .ok_or(BackupStructuralVerificationDenial::VerificationCounterOverflow)?
            .max(structural_peak_owned_allocation_bytes)
            .max(owner_media_open_peak)
            .max(owner_peak_with_retained),
    })
}

fn owner_resource_denial(
    resident: u64,
    required: u64,
    limit: u64,
) -> BackupStructuralVerificationDenial {
    match (resident.checked_add(required), resident.checked_add(limit)) {
        (Some(admitted), Some(limit)) => {
            BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
                phase: BackupVerificationAllocationPhase::OwnerSemanticVerification,
                admitted,
                limit,
            }
        }
        _ => BackupStructuralVerificationDenial::VerificationCounterOverflow,
    }
}
