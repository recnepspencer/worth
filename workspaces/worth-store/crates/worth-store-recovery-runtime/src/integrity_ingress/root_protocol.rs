use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, PhysicalRecordFormatDeclaration, BOOTSTRAP_CATALOG_BYTES,
    ROOT_SELECTOR_BYTES,
};
use worth_store_physical_integrity::{
    validate_bootstrap_catalog, validate_current_root_selector, validate_previous_root_selector,
    validate_root_manifest, CurrentRootSelectorIntegrityValidation, PhysicalArtifactScope,
    PhysicalByteRange, PreviousRootSelectorIntegrityValidation, RootManifestIntegrityValidation,
};

use super::families::root::{
    IntegrityAdmittedCurrentRootSelector, IntegrityAdmittedPreviousRootSelector,
    IntegrityAdmittedRootManifest,
};
use super::routing::rejected_source_binding;
use super::{
    admit_current_root_selector, admit_previous_root_selector, admit_root_manifest,
    observe_absent_recovery_artifact, IntegrityAdmittedRecoveryArtifact, ObservedRecoverySource,
    RecoveryArtifactNamespaceJoin, RecoveryIntegrityIngressAttempt,
    RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) fn admit_observed_bootstrap_catalog<'media>(
    observed: &'media worth_store::physical_runtime::ObservedRecoveryArtifact,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    counters: &mut RecoveryIntegrityIngressCounters,
) -> RecoveryIntegrityIngressAttempt<'media> {
    let scope = PhysicalArtifactScope::bootstrap_catalog(
        store,
        format,
        PhysicalByteRange::new(0, BOOTSTRAP_CATALOG_BYTES as u64)
            .expect("the bootstrap catalog has a nonzero fixed width"),
    );
    if observed.bytes().is_none() {
        return observe_absent_recovery_artifact(observed, scope, counters);
    }
    let source = ObservedRecoverySource::complete(observed, scope);
    let validation = match source.input() {
        Ok(input) => validate_bootstrap_catalog(input, scope).0,
        Err(rejection) => return rejected_source_binding(scope, rejection, counters),
    };
    IntegrityAdmittedRecoveryArtifact::bind_bootstrap_catalog(observed, scope, validation, counters)
}

pub(crate) fn admit_current_selector<'media>(
    join: RecoveryArtifactNamespaceJoin<'media>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<IntegrityAdmittedCurrentRootSelector<'media>, RecoveryIntegrityIngressRejection> {
    let observed = join.require_observed()?;
    let scope = PhysicalArtifactScope::current_root_selector(
        store,
        format,
        PhysicalByteRange::new(0, ROOT_SELECTOR_BYTES as u64)
            .expect("the selector declaration has a nonzero fixed width"),
    );
    let source = ObservedRecoverySource::complete(observed, scope);
    let input = source.input()?;
    let (validation, _) = validate_current_root_selector(input, scope);
    match validation {
        CurrentRootSelectorIntegrityValidation::Intact(validated) => {
            admit_current_root_selector(source, validated)
        }
        CurrentRootSelectorIntegrityValidation::Rejected(rejection) => {
            Err(RecoveryIntegrityIngressRejection::Integrity(rejection))
        }
    }
}

pub(crate) fn admit_previous_selector<'media>(
    join: RecoveryArtifactNamespaceJoin<'media>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<IntegrityAdmittedPreviousRootSelector<'media>, RecoveryIntegrityIngressRejection> {
    let observed = join.require_observed()?;
    let scope = PhysicalArtifactScope::previous_root_selector(
        store,
        format,
        PhysicalByteRange::new(0, ROOT_SELECTOR_BYTES as u64)
            .expect("the selector declaration has a nonzero fixed width"),
    );
    let source = ObservedRecoverySource::complete(observed, scope);
    let input = source.input()?;
    let (validation, _) = validate_previous_root_selector(input, scope);
    match validation {
        PreviousRootSelectorIntegrityValidation::Intact(validated) => {
            admit_previous_root_selector(source, validated)
        }
        PreviousRootSelectorIntegrityValidation::Rejected(rejection) => {
            Err(RecoveryIntegrityIngressRejection::Integrity(rejection))
        }
    }
}

pub(crate) fn admit_addressed_root<'media>(
    join: RecoveryArtifactNamespaceJoin<'media>,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
) -> Result<IntegrityAdmittedRootManifest<'media>, RecoveryIntegrityIngressRejection> {
    let observed = join.require_observed()?;
    let observed_length = observed.bytes().map_or(1, |bytes| bytes.len().max(1)) as u64;
    let scope = PhysicalArtifactScope::root_manifest(
        store,
        format,
        generation,
        PhysicalByteRange::new(0, observed_length)
            .expect("a present root observation has a bounded validation range"),
    )
    .map_err(|_| RecoveryIntegrityIngressRejection::ScopeMismatch)?;
    let source = ObservedRecoverySource::complete(observed, scope);
    let input = source.input()?;
    let (validation, _) = validate_root_manifest(input, scope);
    match validation {
        RootManifestIntegrityValidation::Intact(validated) => {
            admit_root_manifest(source, validated)
        }
        RootManifestIntegrityValidation::Rejected(rejection) => {
            Err(RecoveryIntegrityIngressRejection::Integrity(rejection))
        }
    }
}

pub(crate) fn admit_observed_root_manifest<'media>(
    observed: &'media worth_store::physical_runtime::ObservedRecoveryArtifact,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
    counters: &mut RecoveryIntegrityIngressCounters,
) -> Result<RecoveryIntegrityIngressAttempt<'media>, RecoveryIntegrityIngressRejection> {
    let observed_length = observed.bytes().map_or(1, |bytes| bytes.len().max(1)) as u64;
    let scope = PhysicalArtifactScope::root_manifest(
        store,
        format,
        generation,
        PhysicalByteRange::new(0, observed_length).expect("nonzero observed source width"),
    )
    .map_err(|_| RecoveryIntegrityIngressRejection::ScopeMismatch)?;
    if observed.bytes().is_none() {
        return Ok(observe_absent_recovery_artifact(observed, scope, counters));
    }
    let source = ObservedRecoverySource::complete(observed, scope);
    let validation = match source.input() {
        Ok(input) => validate_root_manifest(input, scope).0,
        Err(rejection) => return Ok(rejected_source_binding(scope, rejection, counters)),
    };
    Ok(IntegrityAdmittedRecoveryArtifact::bind_root_manifest(
        observed, scope, validation, counters,
    ))
}
