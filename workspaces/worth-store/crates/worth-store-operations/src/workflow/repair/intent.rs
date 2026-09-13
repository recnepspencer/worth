use super::integrity_classification::{
    IntegrityRepairArtifactFamily, IntegrityRepairOwnerBinding, IntegrityRepairRegion,
    IntegrityRepairRegionClass,
};
use sha2::{Digest, Sha256};
use worth_store_authority::StoreCurrentAuthorityIdentity;
use worth_store_layout_indexes::DerivedIndexRepairRequest;
use worth_store_offline_verifier::{
    OfflineIntegrityPosture, OperationalTruthRegion, OperationalTruthReport,
};

use crate::{
    OperationalOperationId, OperationalSecurityScope, ProductionRestoreAdmissibleBackupBundle,
};

#[cfg(any(test, feature = "certification-test-authority"))]
mod certification_fixtures;
mod physical_target;
mod region_class;
#[cfg(any(test, feature = "certification-test-authority"))]
pub(crate) use certification_fixtures::{
    certification_authority_repair_candidates_from_backup_observation,
    certification_authority_repair_from_backup_observation,
    certification_derived_maintenance_from_fixture_observation,
};
pub(super) use physical_target::physical_target_identity;
use region_class::repair_class;

#[derive(Debug)]
pub struct RepairIntent {
    operation_id: OperationalOperationId,
    truth: OperationalTruthReport,
    authority_identity: StoreCurrentAuthorityIdentity,
    security_scope: OperationalSecurityScope,
}

impl RepairIntent {
    pub fn from_truth(
        operation_id: OperationalOperationId,
        truth: OperationalTruthReport,
        authority_identity: StoreCurrentAuthorityIdentity,
        security_scope: OperationalSecurityScope,
    ) -> Self {
        Self {
            operation_id,
            truth,
            authority_identity,
            security_scope,
        }
    }

    pub fn resolve(self) -> Result<RepairCandidateSet, RepairResolutionDenial> {
        let mut damaged = Vec::new();
        let mut untouched = 0_u64;
        let mut unrecoverable = Vec::new();
        for region in self.truth.regions() {
            let evidence = region.evidence();
            match region {
                OperationalTruthRegion::AliasGroup { .. }
                | OperationalTruthRegion::OverlapConflict { .. } => {
                    return Err(RepairResolutionDenial::AmbiguousPhysicalOwnership)
                }
                OperationalTruthRegion::TrustedAuthorityRegion(_) => {
                    untouched = untouched
                        .checked_add(1)
                        .ok_or(RepairResolutionDenial::CounterOverflow)?;
                    continue;
                }
                _ => {}
            }
            let class = repair_class(region, evidence.authority_class());
            let identity = region_identity(region);
            let (start, end) = evidence.range();
            let target_identity = physical_target_identity(evidence.source())
                .ok_or(RepairResolutionDenial::InvalidOwnerTarget)?;
            let repair_region = IntegrityRepairRegion::bounded(
                identity,
                start,
                end,
                class,
                evidence.content_digest(),
                target_identity,
                IntegrityRepairOwnerBinding::observed(
                    repair_family(evidence.family()),
                    evidence.generation(),
                    evidence
                        .physical_owner()
                        .map(|owner| owner.stable_fingerprint()),
                    evidence
                        .security_scope()
                        .map(|scope| scope.stable_fingerprint()),
                ),
            )
            .ok_or(RepairResolutionDenial::InvalidRegion)?;
            if class == IntegrityRepairRegionClass::Unrecoverable {
                unrecoverable.push(identity);
            }
            damaged
                .try_reserve(1)
                .map_err(|_| RepairResolutionDenial::AllocationFailed)?;
            damaged.push(super::resolved_region::ResolvedRepairRegion::new(
                repair_region,
                evidence.source().to_path_buf(),
            ));
        }
        if damaged.is_empty() {
            return Err(RepairResolutionDenial::NoRepairRequired);
        }
        damaged.sort();
        let basis_identity = repair_basis_identity(&damaged, self.truth.coverage().covered_bytes());
        Ok(RepairCandidateSet {
            operation_id: self.operation_id,
            damaged,
            untouched,
            unrecoverable,
            basis_identity,
            authority_identity: self.authority_identity,
            security_scope: self.security_scope,
        })
    }
}

const fn repair_family(
    family: worth_store_offline_verifier::OfflinePhysicalArtifactFamily,
) -> IntegrityRepairArtifactFamily {
    match family {
        worth_store_offline_verifier::OfflinePhysicalArtifactFamily::Manifest => {
            IntegrityRepairArtifactFamily::Manifest
        }
        worth_store_offline_verifier::OfflinePhysicalArtifactFamily::Page => {
            IntegrityRepairArtifactFamily::Page
        }
        worth_store_offline_verifier::OfflinePhysicalArtifactFamily::Extent => {
            IntegrityRepairArtifactFamily::Extent
        }
        worth_store_offline_verifier::OfflinePhysicalArtifactFamily::Wal => {
            IntegrityRepairArtifactFamily::Wal
        }
        worth_store_offline_verifier::OfflinePhysicalArtifactFamily::Index => {
            IntegrityRepairArtifactFamily::LayoutIndex
        }
        worth_store_offline_verifier::OfflinePhysicalArtifactFamily::BlobChunk => {
            IntegrityRepairArtifactFamily::BlobChunk
        }
        worth_store_offline_verifier::OfflinePhysicalArtifactFamily::Unknown => {
            IntegrityRepairArtifactFamily::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairResolutionDenial {
    NoRepairRequired,
    AmbiguousPhysicalOwnership,
    InvalidRegion,
    AllocationFailed,
    CounterOverflow,
    NonDerivedDamageRequiresStaging,
    IncompleteOwnerCoverage,
    UnrecoverableDamage,
    IndeterminateDamage,
    StaleTrustedSourceAuthority,
    WrongTrustedSourceSecurityScope,
    InvalidOwnerTarget,
}

#[derive(Debug)]
pub struct RepairCandidateSet {
    pub(super) operation_id: OperationalOperationId,
    pub(super) damaged: Vec<super::resolved_region::ResolvedRepairRegion>,
    pub(super) untouched: u64,
    pub(super) unrecoverable: Vec<[u8; 32]>,
    pub(super) basis_identity: [u8; 32],
    pub(super) authority_identity: StoreCurrentAuthorityIdentity,
    pub(super) security_scope: OperationalSecurityScope,
}

impl RepairCandidateSet {
    pub fn explain(&self) -> RepairPlanExplanation {
        RepairPlanExplanation {
            basis_identity: self.basis_identity,
            damaged_regions: self.damaged.len() as u64,
            untouched_regions: self.untouched,
            unrecoverable_regions: self.unrecoverable.len() as u64,
        }
    }

    pub fn unrecoverable(&self) -> Option<UnrecoverableDamageReport> {
        if self.unrecoverable.is_empty() {
            None
        } else {
            Some(UnrecoverableDamageReport {
                basis_identity: self.basis_identity,
                region_identities: self.unrecoverable.clone(),
            })
        }
    }

    pub fn select_derived_maintenance(
        mut self,
        mut requests: Vec<DerivedIndexRepairRequest>,
    ) -> Result<CurrentAuthorityPreservingMaintenancePlan, RepairResolutionDenial> {
        if !self.unrecoverable.is_empty() {
            return Err(RepairResolutionDenial::UnrecoverableDamage);
        }
        if self.damaged.iter().any(|region| {
            region.integrity().class() != IntegrityRepairRegionClass::DerivedRebuildable
        }) {
            return Err(RepairResolutionDenial::NonDerivedDamageRequiresStaging);
        }
        self.damaged.sort_by_key(|region| {
            let region = region.integrity();
            (region.evidence_digest(), region.target_identity())
        });
        let mut bound_requests = requests
            .drain(..)
            .map(|request| {
                physical_target_identity(request.target())
                    .map(|identity| (request.expected_target_digest(), identity, request))
                    .ok_or(RepairResolutionDenial::InvalidOwnerTarget)
            })
            .collect::<Result<Vec<_>, _>>()?;
        bound_requests.sort_by_key(|(digest, identity, _)| (*digest, *identity));
        if self.damaged.len() != bound_requests.len()
            || self
                .damaged
                .iter()
                .zip(&bound_requests)
                .any(|(region, (digest, identity, _))| {
                    let region = region.integrity();
                    region.evidence_digest() != *digest || region.target_identity() != *identity
                })
        {
            return Err(RepairResolutionDenial::IncompleteOwnerCoverage);
        }
        requests = bound_requests
            .into_iter()
            .map(|(_, _, request)| request)
            .collect();
        Ok(CurrentAuthorityPreservingMaintenancePlan {
            plan: EvidenceBoundRepairPlan {
                operation_id: self.operation_id,
                damaged: super::region_projection::into_integrity_regions(self.damaged)
                    .map_err(|_| RepairResolutionDenial::AllocationFailed)?,
                requests,
                basis_identity: self.basis_identity,
                authority_identity: self.authority_identity,
                security_scope: self.security_scope,
            },
        })
    }

    pub fn select_authority_affecting_staging(
        self,
        backup: ProductionRestoreAdmissibleBackupBundle,
        target_parent: impl Into<std::path::PathBuf>,
        admitted_capacity_bytes: u64,
        copy_buffer_bytes: usize,
    ) -> Result<super::AuthorityAffectingStagedRepairPlan, RepairResolutionDenial> {
        if !self.unrecoverable.is_empty() {
            return Err(RepairResolutionDenial::UnrecoverableDamage);
        }
        if self
            .damaged
            .iter()
            .any(|region| region.integrity().class() == IntegrityRepairRegionClass::Indeterminate)
        {
            return Err(RepairResolutionDenial::IndeterminateDamage);
        }
        if backup.admission().admitting_authority() != self.authority_identity {
            return Err(RepairResolutionDenial::StaleTrustedSourceAuthority);
        }
        let source_scope = backup
            .custody()
            .custody_receipt()
            .identity()
            .stable_fingerprint();
        if self.damaged.iter().any(|region| {
            region
                .integrity()
                .owner_binding()
                .security_scope_identity()
                .is_some_and(|required| required != source_scope)
        }) {
            return Err(RepairResolutionDenial::WrongTrustedSourceSecurityScope);
        }
        Ok(super::AuthorityAffectingStagedRepairPlan::from_resolved(
            self,
            backup,
            target_parent.into(),
            admitted_capacity_bytes,
            copy_buffer_bytes,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepairPlanExplanation {
    basis_identity: [u8; 32],
    damaged_regions: u64,
    untouched_regions: u64,
    unrecoverable_regions: u64,
}

impl RepairPlanExplanation {
    pub const fn basis_identity(self) -> [u8; 32] {
        self.basis_identity
    }
    pub const fn damaged_regions(self) -> u64 {
        self.damaged_regions
    }
    pub const fn intentionally_untouched_regions(self) -> u64 {
        self.untouched_regions
    }
    pub const fn unrecoverable_regions(self) -> u64 {
        self.unrecoverable_regions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrecoverableDamageReport {
    basis_identity: [u8; 32],
    region_identities: Vec<[u8; 32]>,
}

impl UnrecoverableDamageReport {
    pub const fn basis_identity(&self) -> [u8; 32] {
        self.basis_identity
    }
    pub fn region_identities(&self) -> &[[u8; 32]] {
        &self.region_identities
    }
}

#[derive(Debug)]
pub struct CurrentAuthorityPreservingMaintenancePlan {
    pub(super) plan: EvidenceBoundRepairPlan,
}

#[derive(Debug)]
pub struct EvidenceBoundRepairPlan {
    pub(super) operation_id: OperationalOperationId,
    pub(super) damaged: Vec<IntegrityRepairRegion>,
    pub(super) requests: Vec<DerivedIndexRepairRequest>,
    pub(super) basis_identity: [u8; 32],
    pub(super) authority_identity: StoreCurrentAuthorityIdentity,
    pub(super) security_scope: OperationalSecurityScope,
}

fn region_identity(region: &OperationalTruthRegion) -> [u8; 32] {
    let evidence = region.evidence();
    let mut digest = Sha256::new();
    digest.update(b"worth-store-repair-region-v1");
    digest.update(evidence.source().as_os_str().to_string_lossy().as_bytes());
    digest.update(evidence.range().0.to_be_bytes());
    digest.update(evidence.range().1.to_be_bytes());
    digest.update(evidence.content_digest());
    digest.update([match evidence.integrity() {
        OfflineIntegrityPosture::Confirmed => 1,
        OfflineIntegrityPosture::DigestMismatch => 2,
        OfflineIntegrityPosture::IntegrityNotDeclared => 3,
    }]);
    digest.finalize().into()
}

fn repair_basis_identity(
    regions: &[super::resolved_region::ResolvedRepairRegion],
    covered_bytes: u64,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-repair-basis-v1");
    digest.update(covered_bytes.to_be_bytes());
    for region in regions {
        let region = region.integrity();
        digest.update(region.identity());
        digest.update(region.evidence_digest());
    }
    digest.finalize().into()
}
