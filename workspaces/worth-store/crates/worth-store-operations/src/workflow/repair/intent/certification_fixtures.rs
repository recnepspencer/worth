use super::super::integrity_classification::{
    IntegrityRepairArtifactFamily, IntegrityRepairOwnerBinding, IntegrityRepairRegion,
    IntegrityRepairRegionClass,
};
use sha2::{Digest, Sha256};
use worth_store_authority::StoreCurrentAuthorityIdentity;
use worth_store_layout_indexes::DerivedIndexRepairRequest;
use worth_store_physical_format::BackupBundleArtifactFamily;

use crate::{
    OperationalOperationId, OperationalSecurityScope, ProductionRestoreAdmissibleBackupBundle,
};

use super::{
    physical_target_identity, CurrentAuthorityPreservingMaintenancePlan, RepairCandidateSet,
    RepairResolutionDenial,
};

pub(crate) fn certification_derived_maintenance_from_fixture_observation(
    operation_id: OperationalOperationId,
    target: &std::path::Path,
    replacement: &std::path::Path,
    authority_identity: StoreCurrentAuthorityIdentity,
    security_scope: OperationalSecurityScope,
) -> Result<CurrentAuthorityPreservingMaintenancePlan, RepairResolutionDenial> {
    let target_bytes = std::fs::read(target).map_err(|_| RepairResolutionDenial::InvalidRegion)?;
    let replacement_bytes =
        std::fs::read(replacement).map_err(|_| RepairResolutionDenial::InvalidRegion)?;
    let target_digest: [u8; 32] = Sha256::digest(&target_bytes).into();
    let replacement_digest: [u8; 32] = Sha256::digest(&replacement_bytes).into();
    let owner = IntegrityRepairOwnerBinding::observed(
        IntegrityRepairArtifactFamily::LayoutIndex,
        Some(7),
        None,
        None,
    );
    let region = IntegrityRepairRegion::bounded(
        target_digest,
        0,
        target_bytes.len() as u64,
        IntegrityRepairRegionClass::DerivedRebuildable,
        target_digest,
        physical_target_identity(target).ok_or(RepairResolutionDenial::InvalidOwnerTarget)?,
        owner,
    )
    .ok_or(RepairResolutionDenial::InvalidRegion)?;
    let candidates = RepairCandidateSet {
        operation_id,
        damaged: vec![super::super::resolved_region::ResolvedRepairRegion::new(
            region,
            target.to_path_buf(),
        )],
        untouched: 0,
        unrecoverable: Vec::new(),
        basis_identity: target_digest,
        authority_identity,
        security_scope,
    };
    let request = DerivedIndexRepairRequest::new(
        replacement_digest,
        target,
        target_digest,
        replacement,
        replacement_digest,
        7,
        8,
        64 * 1024,
    );
    candidates.select_derived_maintenance(vec![request])
}

pub(crate) fn certification_authority_repair_from_backup_observation(
    operation_id: OperationalOperationId,
    backup: ProductionRestoreAdmissibleBackupBundle,
    target_parent: &std::path::Path,
) -> Result<super::super::AuthorityAffectingStagedRepairPlan, RepairResolutionDenial> {
    certification_authority_repair_candidates_from_backup_observation(operation_id, &backup, None)?
        .select_authority_affecting_staging(backup, target_parent, u64::MAX, 31)
}

pub(crate) fn certification_authority_repair_candidates_from_backup_observation(
    operation_id: OperationalOperationId,
    backup: &ProductionRestoreAdmissibleBackupBundle,
    source_scope_override: Option<[u8; 32]>,
) -> Result<RepairCandidateSet, RepairResolutionDenial> {
    let materialized = backup.custody().structural().materialized();
    let source_root = materialized.root();
    let manifest = materialized.manifest();
    let mut damaged = Vec::with_capacity(manifest.artifacts().len());
    for row in manifest.artifacts().iter().filter(|row| {
        matches!(
            row.family(),
            BackupBundleArtifactFamily::Index | BackupBundleArtifactFamily::BlobChunk
        )
    }) {
        let repair_family = match row.family() {
            BackupBundleArtifactFamily::Index => IntegrityRepairArtifactFamily::LayoutIndex,
            BackupBundleArtifactFamily::BlobChunk => IntegrityRepairArtifactFamily::BlobChunk,
            _ => unreachable!("filter admits only repair-owned artifact families"),
        };
        let source = source_root.join(row.output_name());
        let mut identity = Sha256::new();
        identity.update(b"worth-store-certification-authority-repair-region-v1");
        identity.update([row.family() as u8]);
        identity.update(row.output_name().as_bytes());
        identity.update(row.content_digest());
        let region = IntegrityRepairRegion::bounded(
            identity.finalize().into(),
            0,
            row.bytes(),
            IntegrityRepairRegionClass::QuarantineRequired,
            row.content_digest(),
            physical_target_identity(&source).ok_or(RepairResolutionDenial::InvalidOwnerTarget)?,
            IntegrityRepairOwnerBinding::observed(
                repair_family,
                Some(row.generation()),
                row.reclaim_owner()
                    .generation_owner()
                    .map(|owner| owner.stable_fingerprint()),
                source_scope_override,
            ),
        )
        .ok_or(RepairResolutionDenial::InvalidRegion)?;
        damaged.push(super::super::resolved_region::ResolvedRepairRegion::new(
            region, source,
        ));
    }
    if damaged.is_empty() {
        return Err(RepairResolutionDenial::IncompleteOwnerCoverage);
    }
    let mut basis = Sha256::new();
    basis.update(b"worth-store-certification-authority-repair-basis-v1");
    basis.update(materialized.manifest_digest());
    let damaged_count = damaged.len();
    Ok(RepairCandidateSet {
        operation_id,
        damaged,
        untouched: manifest.artifacts().len().saturating_sub(damaged_count) as u64,
        unrecoverable: Vec::new(),
        basis_identity: basis.finalize().into(),
        authority_identity: backup.admission().admitting_authority(),
        security_scope: OperationalSecurityScope::from_admission(
            backup.custody().custody_receipt(),
        ),
    })
}
