use super::candidate_evaluation::{
    discover_recovery_candidates, RecoveryCandidate, RecoveryCandidateObservation,
    RecoveryCandidateSet,
};

use crate::StructurallyWalkedMedia;

use super::operational_region_composition::{
    canonical_coverage, compose_regions, ordered_files_by_alias, reject_invalid_media_shape,
};
use super::{EvidenceBoundTruthRegion, OfflineTruthEvidenceSet, OperationalTruthRegion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalPhysicalCoverageProof {
    covered_bytes: u64,
    region_count: u64,
}

impl CanonicalPhysicalCoverageProof {
    pub const fn covered_bytes(self) -> u64 {
        self.covered_bytes
    }
    pub const fn region_count(self) -> u64 {
        self.region_count
    }
}

#[derive(Debug)]
pub enum OperationalTruthCompositionDenial {
    DuplicatePhysicalSource,
    EmptyMedia,
    AllocationFailed,
    ConflictingFrontierEvidence,
    CoverageOverflow,
    Interrupted(crate::OfflineInspectionDenial),
    OwnedAllocationBudgetExceeded { admitted: u64, limit: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationalTruthCompositionBudget {
    maximum_owned_allocation_bytes: u64,
}

impl OperationalTruthCompositionBudget {
    pub const fn bounded(maximum_owned_allocation_bytes: u64) -> Option<Self> {
        if maximum_owned_allocation_bytes == 0 {
            None
        } else {
            Some(Self {
                maximum_owned_allocation_bytes,
            })
        }
    }

    pub const fn maximum_owned_allocation_bytes(self) -> u64 {
        self.maximum_owned_allocation_bytes
    }
}

#[derive(Debug)]
pub struct OperationalTruthReport {
    source_inspection_identity: [u8; 32],
    regions: Vec<OperationalTruthRegion>,
    coverage: CanonicalPhysicalCoverageProof,
    candidates: RecoveryCandidateSet,
    peak_owned_allocation_bytes: u64,
}

impl OperationalTruthReport {
    pub const fn source_inspection_identity(&self) -> [u8; 32] {
        self.source_inspection_identity
    }
    pub fn regions(&self) -> &[OperationalTruthRegion] {
        &self.regions
    }
    pub const fn coverage(&self) -> CanonicalPhysicalCoverageProof {
        self.coverage
    }
    pub const fn recovery_candidates(&self) -> &RecoveryCandidateSet {
        &self.candidates
    }
    pub const fn peak_owned_allocation_bytes(&self) -> u64 {
        self.peak_owned_allocation_bytes
    }
}

pub fn compose_operational_truth(
    walked: StructurallyWalkedMedia,
    evidence: &OfflineTruthEvidenceSet,
    budget: OperationalTruthCompositionBudget,
) -> Result<OperationalTruthReport, OperationalTruthCompositionDenial> {
    compose_operational_truth_with_owner_candidates(
        walked,
        evidence,
        Vec::new(),
        budget,
        &mut || Ok(()),
    )
}

pub(crate) fn compose_operational_truth_with_owner_candidates(
    walked: StructurallyWalkedMedia,
    evidence: &OfflineTruthEvidenceSet,
    owner_candidates: Vec<RecoveryCandidateObservation>,
    budget: OperationalTruthCompositionBudget,
    reject_interruption: &mut impl FnMut() -> Result<(), crate::OfflineInspectionDenial>,
) -> Result<OperationalTruthReport, OperationalTruthCompositionDenial> {
    let source_inspection_identity = walked.inspection_evidence_identity();
    reject_invalid_media_shape(&walked, reject_interruption)?;
    let walked_owned_allocation_bytes = walked
        .owned_allocation_bytes()
        .ok_or(OperationalTruthCompositionDenial::CoverageOverflow)?;
    let evidence_owned_allocation_bytes = evidence
        .owned_allocation_bytes()
        .ok_or(OperationalTruthCompositionDenial::CoverageOverflow)?;
    let candidate_input_owned_allocation_bytes =
        allocation_bytes::<RecoveryCandidateObservation>(owner_candidates.capacity())?;
    let maximum_requested_peak = maximum_requested_owned_allocation_bytes(
        &walked,
        evidence_owned_allocation_bytes,
        candidate_input_owned_allocation_bytes,
        owner_candidates.len(),
        reject_interruption,
    )?;
    enforce_owned_allocation_budget(maximum_requested_peak, budget)?;
    let files_by_alias = ordered_files_by_alias(&walked, reject_interruption)?;
    let mut regions = compose_regions(&files_by_alias, evidence, reject_interruption)?;
    let (covered_bytes, region_count) = canonical_coverage(&mut regions, reject_interruption)?;
    let coverage = CanonicalPhysicalCoverageProof {
        covered_bytes,
        region_count,
    };
    reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
    let candidates = discover_recovery_candidates(owner_candidates).map_err(|denial| match denial {
        super::candidate_evaluation::RecoveryCandidateDiscoveryDenial::AllocationFailed => {
            OperationalTruthCompositionDenial::AllocationFailed
        }
        super::candidate_evaluation::RecoveryCandidateDiscoveryDenial::ConflictingFrontierEvidence => {
            OperationalTruthCompositionDenial::ConflictingFrontierEvidence
        }
    })?;
    let actual_peak_owned_allocation_bytes = walked_owned_allocation_bytes
        .checked_add(evidence_owned_allocation_bytes)
        .and_then(|bytes| bytes.checked_add(candidate_input_owned_allocation_bytes))
        .and_then(|bytes| {
            bytes.checked_add(
                allocation_bytes::<&crate::OfflineWalkedFile>(files_by_alias.capacity()).ok()?,
            )
        })
        .and_then(|bytes| bytes.checked_add(regions_owned_allocation_bytes(&regions)?))
        .and_then(|bytes| bytes.checked_add(candidates.owned_allocation_bytes()?))
        .ok_or(OperationalTruthCompositionDenial::CoverageOverflow)?;
    enforce_owned_allocation_budget(actual_peak_owned_allocation_bytes, budget)?;
    Ok(OperationalTruthReport {
        source_inspection_identity,
        coverage,
        regions,
        candidates,
        peak_owned_allocation_bytes: actual_peak_owned_allocation_bytes,
    })
}

fn maximum_requested_owned_allocation_bytes(
    walked: &StructurallyWalkedMedia,
    evidence_owned_allocation_bytes: u64,
    candidate_input_owned_allocation_bytes: u64,
    candidate_count: usize,
    reject_interruption: &mut impl FnMut() -> Result<(), crate::OfflineInspectionDenial>,
) -> Result<u64, OperationalTruthCompositionDenial> {
    let file_count = walked.files().len();
    let mut path_payload_bytes = 0_u64;
    for file in walked.files() {
        reject_interruption().map_err(OperationalTruthCompositionDenial::Interrupted)?;
        path_payload_bytes = path_payload_bytes
            .checked_add(
                path_owned_allocation_bytes(file.path())
                    .ok_or(OperationalTruthCompositionDenial::CoverageOverflow)?,
            )
            .ok_or(OperationalTruthCompositionDenial::CoverageOverflow)?;
    }
    walked
        .owned_allocation_bytes()
        .and_then(|bytes| bytes.checked_add(evidence_owned_allocation_bytes))
        .and_then(|bytes| bytes.checked_add(candidate_input_owned_allocation_bytes))
        .and_then(|bytes| {
            bytes.checked_add(allocation_bytes::<&crate::OfflineWalkedFile>(file_count).ok()?)
        })
        .and_then(|bytes| {
            bytes.checked_add(allocation_bytes::<OperationalTruthRegion>(file_count).ok()?)
        })
        .and_then(|bytes| {
            bytes.checked_add(allocation_bytes::<EvidenceBoundTruthRegion>(file_count).ok()?)
        })
        .and_then(|bytes| {
            bytes.checked_add(allocation_bytes::<std::path::PathBuf>(file_count).ok()?)
        })
        .and_then(|bytes| bytes.checked_add(path_payload_bytes.checked_mul(2)?))
        .and_then(|bytes| {
            bytes.checked_add(allocation_bytes::<RecoveryCandidate>(candidate_count).ok()?)
        })
        .ok_or(OperationalTruthCompositionDenial::CoverageOverflow)
}

fn regions_owned_allocation_bytes(regions: &Vec<OperationalTruthRegion>) -> Option<u64> {
    regions.iter().try_fold(
        allocation_bytes::<OperationalTruthRegion>(regions.capacity()).ok()?,
        |total, region| {
            let with_region =
                total.checked_add(path_owned_allocation_bytes(region.evidence().source())?)?;
            match region {
                OperationalTruthRegion::AliasGroup { claimants, .. } => claimants.iter().try_fold(
                    with_region.checked_add(
                        allocation_bytes::<std::path::PathBuf>(claimants.capacity()).ok()?,
                    )?,
                    |bytes, claimant| bytes.checked_add(path_owned_allocation_bytes(claimant)?),
                ),
                OperationalTruthRegion::OverlapConflict {
                    additional_claims,
                    claimants,
                    ..
                } => {
                    let with_claim_rows = with_region.checked_add(
                        allocation_bytes::<EvidenceBoundTruthRegion>(additional_claims.capacity())
                            .ok()?,
                    )?;
                    let with_claim_evidence =
                        additional_claims
                            .iter()
                            .try_fold(with_claim_rows, |bytes, claim| {
                                bytes.checked_add(path_owned_allocation_bytes(claim.source())?)
                            })?;
                    claimants.iter().try_fold(
                        with_claim_evidence.checked_add(
                            allocation_bytes::<std::path::PathBuf>(claimants.capacity()).ok()?,
                        )?,
                        |bytes, claimant| bytes.checked_add(path_owned_allocation_bytes(claimant)?),
                    )
                }
                _ => Some(with_region),
            }
        },
    )
}

fn enforce_owned_allocation_budget(
    admitted: u64,
    budget: OperationalTruthCompositionBudget,
) -> Result<(), OperationalTruthCompositionDenial> {
    if admitted > budget.maximum_owned_allocation_bytes() {
        Err(
            OperationalTruthCompositionDenial::OwnedAllocationBudgetExceeded {
                admitted,
                limit: budget.maximum_owned_allocation_bytes(),
            },
        )
    } else {
        Ok(())
    }
}

fn allocation_bytes<T>(capacity: usize) -> Result<u64, OperationalTruthCompositionDenial> {
    u64::try_from(capacity)
        .ok()
        .and_then(|count| count.checked_mul(std::mem::size_of::<T>() as u64))
        .ok_or(OperationalTruthCompositionDenial::CoverageOverflow)
}

#[cfg(windows)]
fn path_owned_allocation_bytes(path: &std::path::Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    u64::try_from(path.as_os_str().encode_wide().count())
        .ok()?
        .checked_mul(2)
}

#[cfg(unix)]
fn path_owned_allocation_bytes(path: &std::path::Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    u64::try_from(path.as_os_str().as_bytes().len()).ok()
}

#[cfg(not(any(windows, unix)))]
fn path_owned_allocation_bytes(path: &std::path::Path) -> Option<u64> {
    u64::try_from(path.to_string_lossy().len()).ok()
}
