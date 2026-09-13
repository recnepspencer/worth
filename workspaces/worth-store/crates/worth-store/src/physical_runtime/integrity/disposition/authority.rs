use worth_store_buffer_pool::PhysicalFrameLease;
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::RecordArtifactFile;
use worth_store_physical_integrity::{
    validate_current_root_selector, validate_previous_root_selector, validate_root_manifest,
    CurrentRootSelectorIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
    PhysicalDamageLocalization, PhysicalIntegrityRejection,
    PreviousRootSelectorIntegrityValidation, RootManifestIntegrityValidation,
    UntrustedPhysicalArtifact,
};

use super::classification::{
    project_damaged_authority, project_intact_authority, project_rejection_without_owner_truth,
    OwnerDispositionProjectionDenial, PhysicalArtifactDisposition,
};

/// Internal owner role joined only while validating one exact C.6 source.
pub(super) struct StoreAuthoritativeArtifactOwnerTruth {
    scope: PhysicalArtifactScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum StoreOwnerDispositionAdapterDenial {
    SourceScopeSubstitution,
    Projection(OwnerDispositionProjectionDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntactPhysicalAuthorityObservation {
    scope: PhysicalArtifactScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamagedPhysicalAuthorityObservation {
    localization: PhysicalDamageLocalization,
}

/// Validate and project the inherited Phase 3 current-selector owner source.
pub(in crate::physical_runtime) fn project_resident_current_root_selector_authority(
    source: &PhysicalFrameLease,
    scope: PhysicalArtifactScope,
) -> Result<PhysicalArtifactDisposition, StoreOwnerDispositionAdapterDenial> {
    let owner_truth = bind_resident_source(
        source,
        scope,
        PhysicalIntegrityArtifactFamily::CurrentRootSelector,
        RecordArtifactFile::CurrentRootSelector,
    )?;
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(source);
    match validate_current_root_selector(input, scope).0 {
        CurrentRootSelectorIntegrityValidation::Intact(validated) => {
            project_intact_authority(owner_truth, validated.into_validation_record())
        }
        CurrentRootSelectorIntegrityValidation::Rejected(rejection) => {
            project_authoritative_rejection(owner_truth, rejection)
        }
    }
    .map_err(StoreOwnerDispositionAdapterDenial::Projection)
}

/// Validate and project the inherited Phase 3 previous-selector owner source.
pub(in crate::physical_runtime) fn project_resident_previous_root_selector_authority(
    source: &PhysicalFrameLease,
    scope: PhysicalArtifactScope,
) -> Result<PhysicalArtifactDisposition, StoreOwnerDispositionAdapterDenial> {
    let owner_truth = bind_resident_source(
        source,
        scope,
        PhysicalIntegrityArtifactFamily::PreviousRootSelector,
        RecordArtifactFile::PreviousRootSelector,
    )?;
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(source);
    match validate_previous_root_selector(input, scope).0 {
        PreviousRootSelectorIntegrityValidation::Intact(validated) => {
            project_intact_authority(owner_truth, validated.into_validation_record())
        }
        PreviousRootSelectorIntegrityValidation::Rejected(rejection) => {
            project_authoritative_rejection(owner_truth, rejection)
        }
    }
    .map_err(StoreOwnerDispositionAdapterDenial::Projection)
}

/// Validate and project the inherited Phase 3 root-manifest owner source.
pub(in crate::physical_runtime) fn project_resident_root_manifest_authority(
    source: &PhysicalFrameLease,
    scope: PhysicalArtifactScope,
) -> Result<PhysicalArtifactDisposition, StoreOwnerDispositionAdapterDenial> {
    let generation = scope
        .root_generation()
        .ok_or(StoreOwnerDispositionAdapterDenial::SourceScopeSubstitution)?;
    let owner_truth = bind_resident_source(
        source,
        scope,
        PhysicalIntegrityArtifactFamily::RootManifest,
        RecordArtifactFile::RootManifest { generation },
    )?;
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(source);
    match validate_root_manifest(input, scope).0 {
        RootManifestIntegrityValidation::Intact(validated) => {
            project_intact_authority(owner_truth, validated.into_validation_record())
        }
        RootManifestIntegrityValidation::Rejected(rejection) => {
            project_authoritative_rejection(owner_truth, rejection)
        }
    }
    .map_err(StoreOwnerDispositionAdapterDenial::Projection)
}

fn bind_resident_source(
    source: &PhysicalFrameLease,
    scope: PhysicalArtifactScope,
    expected_family: PhysicalIntegrityArtifactFamily,
    expected_artifact: RecordArtifactFile,
) -> Result<StoreAuthoritativeArtifactOwnerTruth, StoreOwnerDispositionAdapterDenial> {
    let key = source.key();
    let coordinate = key.coordinate();
    let expected_range =
        PhysicalByteRange::new(coordinate.offset(), u64::from(coordinate.length()))
            .expect("an admitted resident-frame coordinate has a nonempty range");
    if key.store() != scope.store_identity()
        || scope.artifact_family() != expected_family
        || coordinate.artifact() != expected_artifact
        || scope.byte_range() != expected_range
    {
        return Err(StoreOwnerDispositionAdapterDenial::SourceScopeSubstitution);
    }
    Ok(StoreAuthoritativeArtifactOwnerTruth { scope })
}

fn project_authoritative_rejection(
    owner_truth: StoreAuthoritativeArtifactOwnerTruth,
    rejection: PhysicalIntegrityRejection,
) -> Result<PhysicalArtifactDisposition, OwnerDispositionProjectionDenial> {
    if matches!(rejection, PhysicalIntegrityRejection::Damaged(_)) {
        project_damaged_authority(owner_truth, rejection)
    } else {
        project_rejection_without_owner_truth(rejection)
    }
}

impl StoreAuthoritativeArtifactOwnerTruth {
    pub(super) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }
}

impl IntactPhysicalAuthorityObservation {
    pub(super) const fn new(scope: PhysicalArtifactScope) -> Self {
        Self { scope }
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }
}

impl DamagedPhysicalAuthorityObservation {
    pub(super) const fn new(localization: PhysicalDamageLocalization) -> Self {
        Self { localization }
    }

    pub const fn localization(self) -> PhysicalDamageLocalization {
        self.localization
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.localization.scope()
    }
}
