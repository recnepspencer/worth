use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy,
    PhysicalManifestCapacityTransition, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationDenial, PhysicalPageSizeClass, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordOpen, RecordAppendBatch, RecordAppendDenial,
    RecordBootstrapDenial,
};

pub(super) struct InvalidScaleWorlds {
    pub(super) missing_catalog_refused: bool,
    pub(super) checksum_damage_refused: bool,
    pub(super) stale_manifest_refused: bool,
    pub(super) format_drift_refused: bool,
    pub(super) residue_excluded: bool,
}

impl InvalidScaleWorlds {
    pub(super) fn count(&self) -> u8 {
        [
            self.missing_catalog_refused,
            self.checksum_damage_refused,
            self.stale_manifest_refused,
            self.format_drift_refused,
            self.residue_excluded,
        ]
        .into_iter()
        .filter(|passed| *passed)
        .count() as u8
    }
}

pub(super) fn exercise(
    source: &Path,
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    access: AdmittedRecordAccessPolicy,
) -> InvalidScaleWorlds {
    InvalidScaleWorlds {
        missing_catalog_refused: missing_catalog(source, format, access),
        checksum_damage_refused: checksum_damage(source, format, access),
        stale_manifest_refused: stale_manifest(source, format, access),
        format_drift_refused: format_drift(source),
        residue_excluded: unpublished_residue(source, format, placement, access),
    }
}

fn missing_catalog(
    source: &Path,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
) -> bool {
    let catalog = source.join("families/records/bootstrap.catalog");
    let original = std::fs::read(&catalog).unwrap();
    std::fs::remove_file(&catalog).unwrap();
    let TransitionOutcome::Denied(denial) = open_record_store!(
        super::media(source),
        |durability| PhysicalRecordOpen::new(format, access, durability)
    )
    .into_raw() else {
        std::fs::write(catalog, original).unwrap();
        return false;
    };
    let matched = denial.reason() == RecordBootstrapDenial::AmbiguousRecordFamilyResidue
        && worth_store_offline_verifier::walk_current_durable_record_manifest(
            source,
            format.declaration(),
        )
        .is_err();
    denial.into_runtime().close();
    std::fs::write(catalog, original).unwrap();
    matched
}

fn checksum_damage(
    source: &Path,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
) -> bool {
    let root = current_root_path(source, format);
    let original = std::fs::read(&root).unwrap();
    let mut bytes = original.clone();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    std::fs::write(&root, bytes).unwrap();
    let TransitionOutcome::Denied(denial) = open_record_store!(
        super::media(source),
        |durability| PhysicalRecordOpen::new(format, access, durability)
    )
    .into_raw() else {
        std::fs::write(root, original).unwrap();
        return false;
    };
    let matched = denial.reason() == RecordBootstrapDenial::CurrentRootDamaged
        && worth_store_offline_verifier::walk_current_durable_record_manifest(
            source,
            format.declaration(),
        )
        .is_err();
    denial.into_runtime().close();
    std::fs::write(root, original).unwrap();
    matched
}

fn stale_manifest(
    source: &Path,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
) -> bool {
    let current = current_root_path(source, format);
    let original = std::fs::read(&current).unwrap();
    let stale = source.join("families/records/roots/root-0000000000000001.manifest");
    std::fs::copy(stale, &current).unwrap();
    let matched =
        match open_record_store!(super::media(source), |durability| PhysicalRecordOpen::new(
            format, access, durability
        ))
        .into_raw()
        {
            TransitionOutcome::Denied(denial) => {
                // Substituting an old image at the selected root's address is
                // persisted generation damage, not a stale caller request.
                let damaged = denial.reason() == RecordBootstrapDenial::CurrentRootDamaged;
                denial.into_runtime().close();
                damaged
                    && worth_store_offline_verifier::walk_current_durable_record_manifest(
                        source,
                        format.declaration(),
                    )
                    .is_err()
            }
            TransitionOutcome::Success(runtime) => {
                runtime.close();
                false
            }
            TransitionOutcome::Stale(stale) => {
                stale.into_runtime().close();
                false
            }
            _ => false,
        };
    std::fs::write(current, original).unwrap();
    matched
}

fn format_drift(source: &Path) -> bool {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let TransitionOutcome::Denied(denial) = open_record_store!(
        super::media(source),
        |durability| PhysicalRecordOpen::new(format, access, durability)
    )
    .into_raw() else {
        return false;
    };
    let matched = matches!(
        denial.reason(),
        RecordBootstrapDenial::PhysicalRecordFormatMismatch(_)
    ) && worth_store_offline_verifier::walk_current_durable_record_manifest(
        source,
        format.declaration(),
    )
    .is_err();
    denial.into_runtime().close();
    matched
}

fn unpublished_residue(
    source: &Path,
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    access: AdmittedRecordAccessPolicy,
) -> bool {
    let catalog = source.join("families/records/bootstrap.catalog");
    let candidate = source.join("staging/records/bootstrap-deadbeefdeadbeef.candidate");
    let candidate_bytes = std::fs::copy(&catalog, &candidate).unwrap();
    assert_eq!(candidate_bytes, std::fs::metadata(catalog).unwrap().len());
    let reopened = super::success(open_record_store!(super::media(source), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let excluded = reopened.observed_staging_residue()
        && reopened.publication_residue().staging_catalog_candidate()
        && reopened.observed_non_authoritative_residue()
        && matches!(
            super::durable_publication::prepare_single(
                &reopened.record_submission(),
                placement,
                PhysicalManifestCapacityTransition::PreserveCurrent,
                PhysicalMutationIdempotencyMaterial::new([210; 32]),
                RecordAppendBatch::try_from_iter([b"blocked".as_slice()]).unwrap(),
            )
            .into_raw(),
            TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
                RecordAppendDenial::ServingRequiresInspection
            ))
        );
    reopened.close();
    let offline_prior = worth_store_offline_verifier::walk_current_durable_record_manifest(
        source,
        format.declaration(),
    )
    .is_ok_and(|walk| !walk.placements().is_empty());
    std::fs::remove_file(candidate).unwrap();
    excluded && offline_prior
}

fn current_root_path(root: &Path, format: AdmittedPhysicalRecordFormat) -> std::path::PathBuf {
    let generation = worth_store_offline_verifier::walk_current_durable_record_manifest(
        root,
        format.declaration(),
    )
    .unwrap()
    .root_generation();
    root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    ))
}
