use worth_store_physical_backend::{
    OfflineMediaClosureEntry, OfflineMediaConsistencyBasis, OfflineMediaConsistencyBasisDenial,
    OfflineMediaReadDenial,
};
use worth_store_physical_isolation::BackupCutManifest;

use crate::{OfflineInspectionBudget, OfflineInspectionCancellation, OfflineInspectionDenial};

use super::owner_artifact_verification::verify_owner_artifact;
use super::owner_media_read::OwnerMediaReadSession;
use super::owner_semantic_verification::OwnerSemanticVerificationCounters;
use super::verification_owned_memory::{allocation_bytes, defect_owned_allocation_bytes};
use super::{BackupVerificationDefect, BackupVerificationReadAccounting};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupCutSourceVerificationReport {
    defects: Vec<BackupVerificationDefect>,
    artifacts_attempted: u64,
    artifacts_verified: u64,
    admitted_read_bytes: u64,
    inspected_bytes: u64,
    verified_bytes: u64,
    decoder_allocation_bytes: u64,
    peak_buffer_bytes: u64,
    peak_owned_allocation_bytes: u64,
    read_accounting: BackupVerificationReadAccounting,
}

impl BackupCutSourceVerificationReport {
    pub fn defects(&self) -> &[BackupVerificationDefect] {
        &self.defects
    }
    pub const fn artifacts_attempted(&self) -> u64 {
        self.artifacts_attempted
    }
    pub const fn artifacts_verified(&self) -> u64 {
        self.artifacts_verified
    }
    pub const fn admitted_read_bytes(&self) -> u64 {
        self.admitted_read_bytes
    }
    pub const fn bytes_verified(&self) -> u64 {
        self.verified_bytes
    }
    pub const fn inspected_bytes(&self) -> u64 {
        self.inspected_bytes
    }
    pub const fn decoder_allocation_bytes(&self) -> u64 {
        self.decoder_allocation_bytes
    }
    pub const fn peak_buffer_bytes(&self) -> u64 {
        self.peak_buffer_bytes
    }
    pub const fn peak_owned_allocation_bytes(&self) -> u64 {
        self.peak_owned_allocation_bytes
    }
    pub const fn read_accounting(&self) -> BackupVerificationReadAccounting {
        self.read_accounting
    }
}

#[derive(Debug)]
pub enum BackupCutSourceVerificationDenial {
    ReadBudgetExceeded { required: u64, limit: u64 },
    OwnedAllocationBudgetExceeded { required: u64, limit: u64 },
    AllocationFailed,
    ConsistencyBasis(OfflineMediaConsistencyBasisDenial),
    Media(OfflineMediaReadDenial),
    Inspection(OfflineInspectionDenial),
    CounterOverflow,
    Defects(BackupCutSourceVerificationReport),
}

pub fn verify_backup_cut_sources(
    manifest: &BackupCutManifest,
    budget: OfflineInspectionBudget,
) -> Result<BackupCutSourceVerificationReport, BackupCutSourceVerificationDenial> {
    verify_backup_cut_sources_with_cancellation(
        manifest,
        budget,
        OfflineInspectionCancellation::new(),
    )
}

pub fn verify_backup_cut_sources_with_cancellation(
    manifest: &BackupCutManifest,
    budget: OfflineInspectionBudget,
    cancellation: OfflineInspectionCancellation,
) -> Result<BackupCutSourceVerificationReport, BackupCutSourceVerificationDenial> {
    let started_at = std::time::Instant::now();
    reject_interruption(budget, &cancellation, started_at)?;
    let required = manifest.total_bytes();
    if required > budget.max_total_read_bytes() {
        return Err(BackupCutSourceVerificationDenial::ReadBudgetExceeded {
            required,
            limit: budget.max_total_read_bytes(),
        });
    }
    let basis = source_consistency_basis(manifest, budget, &cancellation, started_at)?;
    let mut media = OwnerMediaReadSession::open(
        manifest
            .artifacts()
            .iter()
            .map(|artifact| artifact.source_path().to_path_buf()),
        basis,
        budget,
        cancellation,
        started_at,
    )
    .map_err(BackupCutSourceVerificationDenial::Media)?;
    media
        .reject_interruption()
        .map_err(BackupCutSourceVerificationDenial::Inspection)?;
    let maximum_defect_bytes = maximum_defect_owned_bytes(manifest, &media)?;
    let working_peak = media
        .resident_owned_allocation_bytes()
        .checked_add(maximum_defect_bytes)
        .and_then(|bytes| bytes.checked_add(budget.max_buffer_bytes() as u64))
        .ok_or(BackupCutSourceVerificationDenial::CounterOverflow)?;
    let admitted_peak = media.peak_owned_allocation_bytes().max(working_peak);
    if admitted_peak > budget.maximum_owned_allocation_bytes() {
        return Err(
            BackupCutSourceVerificationDenial::OwnedAllocationBudgetExceeded {
                required: admitted_peak,
                limit: budget.maximum_owned_allocation_bytes(),
            },
        );
    }
    let mut defects = Vec::new();
    defects
        .try_reserve_exact(manifest.artifacts().len())
        .map_err(|_| BackupCutSourceVerificationDenial::AllocationFailed)?;
    let mut counters = OwnerSemanticVerificationCounters::default();
    let mut expected_root = None;
    for (index, artifact) in manifest.artifacts().iter().enumerate() {
        media
            .reject_interruption()
            .map_err(BackupCutSourceVerificationDenial::Inspection)?;
        counters = counters
            .record_attempt()
            .ok_or(BackupCutSourceVerificationDenial::CounterOverflow)?;
        let output_name = format!("source-{index:04}");
        let Some(row) = artifact.portable_manifest_row(&output_name) else {
            defects.push(BackupVerificationDefect::OwnerSemanticMismatch {
                output_name,
                format: artifact.format(),
                kind: super::BackupArtifactSemanticDefectKind::OwnerReferenceInvalid,
            });
            continue;
        };
        let mut reader = media
            .reader(artifact.source_path())
            .map_err(BackupCutSourceVerificationDenial::Media)?;
        let actual_bytes = reader.length();
        let verification = verify_owner_artifact(
            &mut reader,
            actual_bytes,
            expected_root,
            &row,
            budget.max_buffer_bytes(),
        );
        let bytes_read = reader
            .finish()
            .map_err(BackupCutSourceVerificationDenial::Inspection)?;
        counters = counters
            .record_read(bytes_read)
            .ok_or(BackupCutSourceVerificationDenial::CounterOverflow)?;
        match verification {
            Ok(verified) => {
                if verified.root_publication().is_some() {
                    expected_root = verified.root_publication();
                }
                counters = counters
                    .record(verified.observation())
                    .ok_or(BackupCutSourceVerificationDenial::CounterOverflow)?;
            }
            Err(kind) => defects.push(BackupVerificationDefect::OwnerSemanticMismatch {
                output_name,
                format: artifact.format(),
                kind,
            }),
        }
    }
    media
        .revalidate_consistency()
        .map_err(BackupCutSourceVerificationDenial::Media)?;
    let defect_bytes = defect_owned_allocation_bytes(&defects)
        .ok_or(BackupCutSourceVerificationDenial::CounterOverflow)?;
    let peak_owned_allocation_bytes = media.peak_owned_allocation_bytes().max(
        media
            .resident_owned_allocation_bytes()
            .checked_add(defect_bytes)
            .and_then(|bytes| bytes.checked_add(budget.max_buffer_bytes() as u64))
            .ok_or(BackupCutSourceVerificationDenial::CounterOverflow)?,
    );
    let report = BackupCutSourceVerificationReport {
        defects,
        artifacts_attempted: counters.artifacts_attempted(),
        artifacts_verified: counters.artifacts_verified(),
        admitted_read_bytes: required,
        inspected_bytes: counters.bytes_read(),
        verified_bytes: counters.bytes_verified(),
        decoder_allocation_bytes: counters.decoder_allocation_bytes(),
        peak_buffer_bytes: counters.peak_buffer_bytes(),
        peak_owned_allocation_bytes,
        read_accounting: BackupVerificationReadAccounting::Complete,
    };
    if report.defects.is_empty() {
        Ok(report)
    } else {
        Err(BackupCutSourceVerificationDenial::Defects(report))
    }
}

fn source_consistency_basis(
    manifest: &BackupCutManifest,
    budget: OfflineInspectionBudget,
    cancellation: &OfflineInspectionCancellation,
    started_at: std::time::Instant,
) -> Result<OfflineMediaConsistencyBasis, BackupCutSourceVerificationDenial> {
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(manifest.artifacts().len())
        .map_err(|_| BackupCutSourceVerificationDenial::AllocationFailed)?;
    for artifact in manifest.artifacts() {
        reject_interruption(budget, cancellation, started_at)?;
        entries.push(
            OfflineMediaClosureEntry::new(
                artifact.source_path(),
                artifact.bytes(),
                artifact.content_digest(),
            )
            .ok_or(BackupCutSourceVerificationDenial::AllocationFailed)?,
        );
    }
    OfflineMediaConsistencyBasis::content_addressed_closure_from_owned_entries(
        hex(&manifest.artifact_closure_digest()),
        entries,
    )
    .map_err(BackupCutSourceVerificationDenial::ConsistencyBasis)
}

fn maximum_defect_owned_bytes(
    manifest: &BackupCutManifest,
    media: &OwnerMediaReadSession,
) -> Result<u64, BackupCutSourceVerificationDenial> {
    let mut total = allocation_bytes::<BackupVerificationDefect>(manifest.artifacts().len())
        .ok_or(BackupCutSourceVerificationDenial::CounterOverflow)?;
    for (index, _) in manifest.artifacts().iter().enumerate() {
        media
            .reject_interruption()
            .map_err(BackupCutSourceVerificationDenial::Inspection)?;
        total = total
            .checked_add(
                source_output_name_bytes(index)
                    .ok_or(BackupCutSourceVerificationDenial::CounterOverflow)?,
            )
            .ok_or(BackupCutSourceVerificationDenial::CounterOverflow)?;
    }
    Ok(total)
}

fn reject_interruption(
    budget: OfflineInspectionBudget,
    cancellation: &OfflineInspectionCancellation,
    started_at: std::time::Instant,
) -> Result<(), BackupCutSourceVerificationDenial> {
    crate::inspection::reject_inspection_interruption(budget, cancellation, started_at)
        .map_err(BackupCutSourceVerificationDenial::Inspection)
}

fn source_output_name_bytes(index: usize) -> Option<u64> {
    let digits = if index < 10_000 {
        4
    } else {
        index.checked_ilog10()?.checked_add(1)? as usize
    };
    u64::try_from("source-".len().checked_add(digits)?).ok()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
