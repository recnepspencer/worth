pub(crate) use std::path::Path;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordAccessPolicy,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalManifestCapacityTransition,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationDenial,
    worth_store::physical_runtime::PhysicalPageSizeClass,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordAppendDenial,
    worth_store::physical_runtime::RecordBootstrapDenial,
};

pub(crate) struct InvalidScaleWorlds {
    pub(crate) missing_catalog_refused: bool,
    pub(crate) checksum_damage_refused: bool,
    pub(crate) stale_manifest_refused: bool,
    pub(crate) format_drift_refused: bool,
    pub(crate) residue_excluded: bool,
}

impl InvalidScaleWorlds {
    pub(crate) fn count(&self) -> u8 {
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

pub(crate) fn exercise(
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

pub(crate) fn missing_catalog(
    source: &Path,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
) -> bool {
    let catalog = source.join("families/records/bootstrap.catalog");
    let original = std::fs::read(&catalog).unwrap();
    std::fs::remove_file(&catalog).unwrap();
    let worth_proof::TransitionOutcome::Denied(denial) = open_record_store!(
        worth_store_testkit_store::physical_record_journeys::media(source),
        |durability| worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability
        )
    )
    .into_raw() else {
        std::fs::write(catalog, original).unwrap();
        return false;
    };
    let matched = denial.reason()
        == worth_store::physical_runtime::RecordBootstrapDenial::AmbiguousRecordFamilyResidue
        && worth_store_offline_verifier::walk_current_durable_record_manifest(
            source,
            format.declaration(),
        )
        .is_err();
    denial.into_runtime().close();
    std::fs::write(catalog, original).unwrap();
    matched
}

pub(crate) fn checksum_damage(
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
    let worth_proof::TransitionOutcome::Denied(denial) = open_record_store!(
        worth_store_testkit_store::physical_record_journeys::media(source),
        |durability| worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability
        )
    )
    .into_raw() else {
        std::fs::write(root, original).unwrap();
        return false;
    };
    let matched = denial.reason()
        == worth_store::physical_runtime::RecordBootstrapDenial::CurrentRootDamaged
        && worth_store_offline_verifier::walk_current_durable_record_manifest(
            source,
            format.declaration(),
        )
        .is_err();
    denial.into_runtime().close();
    std::fs::write(root, original).unwrap();
    matched
}

pub(crate) fn stale_manifest(
    source: &Path,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
) -> bool {
    let current = current_root_path(source, format);
    let original = std::fs::read(&current).unwrap();
    let stale = source.join("families/records/roots/root-0000000000000001.manifest");
    std::fs::copy(stale, &current).unwrap();
    let matched = match open_record_store!(
        worth_store_testkit_store::physical_record_journeys::media(source),
        |durability| worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability
        )
    )
    .into_raw()
    {
        worth_proof::TransitionOutcome::Denied(denial) => {
            // Substituting an old image at the selected root's address is
            // persisted generation damage, not a stale caller request.
            let damaged = denial.reason()
                == worth_store::physical_runtime::RecordBootstrapDenial::CurrentRootDamaged;
            denial.into_runtime().close();
            damaged
                && worth_store_offline_verifier::walk_current_durable_record_manifest(
                    source,
                    format.declaration(),
                )
                .is_err()
        }
        worth_proof::TransitionOutcome::Success(runtime) => {
            runtime.close();
            false
        }
        worth_proof::TransitionOutcome::Stale(stale) => {
            stale.into_runtime().close();
            false
        }
        _ => false,
    };
    std::fs::write(current, original).unwrap();
    matched
}

pub(crate) fn format_drift(source: &Path) -> bool {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .page_size(worth_store::physical_runtime::PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let worth_proof::TransitionOutcome::Denied(denial) = open_record_store!(
        worth_store_testkit_store::physical_record_journeys::media(source),
        |durability| worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability
        )
    )
    .into_raw() else {
        return false;
    };
    let matched = matches!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::PhysicalRecordFormatMismatch(_)
    ) && worth_store_offline_verifier::walk_current_durable_record_manifest(
        source,
        format.declaration(),
    )
    .is_err();
    denial.into_runtime().close();
    matched
}

pub(crate) fn unpublished_residue(
    source: &Path,
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    access: AdmittedRecordAccessPolicy,
) -> bool {
    let catalog = source.join("families/records/bootstrap.catalog");
    let candidate = source.join("staging/records/bootstrap-deadbeefdeadbeef.candidate");
    let candidate_bytes = std::fs::copy(&catalog, &candidate).unwrap();
    assert_eq!(candidate_bytes, std::fs::metadata(catalog).unwrap().len());
    let reopened =
        worth_store_testkit_store::physical_record_journeys::success(open_record_store!(
            worth_store_testkit_store::physical_record_journeys::media(source),
            |durability| {
                worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            }
        ));
    let excluded = reopened.observed_staging_residue()
        && reopened.publication_residue().staging_catalog_candidate()
        && reopened.observed_non_authoritative_residue()
        && matches!(
            worth_store_testkit_store::durable_publication::wal_data::prepare_single(
                &reopened.record_submission(),
                placement,
                worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
                worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([210; 32]),
                worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                    b"blocked".as_slice()
                ])
                .unwrap(),
            )
            .into_raw(),
            worth_proof::TransitionOutcome::Denied(
                worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
                    worth_store::physical_runtime::RecordAppendDenial::ServingRequiresInspection
                )
            )
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

pub(crate) fn current_root_path(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
) -> std::path::PathBuf {
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
