use crate::strategy::registry::{
    layout_admission_registry, LayoutAdmissionDenial, LayoutAdmissionRequest,
    LayoutRequestedCapability, LayoutStrategyCapability,
};
use crate::strategy::tests_support::{admit_strategy_scope, root_manifest_scope};
use crate::strategy::LayoutStrategyFamily;
use crate::{ArtifactFamilyAccessLane, IndexMaintenanceMode, PhysicalMutationShape};
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_security::{
    StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
    StoreKeyScope, StoreTenantScope,
};

#[test]
fn registry_admits_supported_requests_to_stable_owner_snapshots() {
    let (lifecycle, domain) = page_scope();
    let request = LayoutAdmissionRequest::from_admitted(
        lifecycle,
        domain,
        LayoutStrategyFamily::BTreeRange,
        LayoutRequestedCapability::point_lookup(),
        ArtifactFamilyAccessLane::HotPath,
    );

    let first = layout_admission_registry()
        .admit(request.clone())
        .into_result()
        .unwrap();
    let second = layout_admission_registry()
        .admit(request)
        .into_result()
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(
        first.admitted_strategy().family(),
        LayoutStrategyFamily::BTreeRange
    );
    assert_eq!(
        first.granted_capability(),
        LayoutStrategyCapability::PointLookup
    );
}

#[test]
fn registry_denies_unsupported_capability_and_scope_mismatch() {
    let (lifecycle, domain) = page_scope();
    let (_, root_domain) = root_manifest_scope();
    let unsupported = LayoutAdmissionRequest::from_admitted(
        lifecycle,
        domain,
        LayoutStrategyFamily::BTreeRange,
        LayoutRequestedCapability::blob_streaming(),
        ArtifactFamilyAccessLane::HotPath,
    );
    let scope_mismatch = LayoutAdmissionRequest::from_admitted(
        lifecycle,
        domain,
        LayoutStrategyFamily::BTreeRange,
        LayoutRequestedCapability::point_lookup(),
        ArtifactFamilyAccessLane::HotPath,
    )
    .within_scope_partition(root_domain.witness().scope());

    assert!(matches!(
        layout_admission_registry().admit(unsupported).into_result(),
        Err(LayoutAdmissionDenial::StrategyDoesNotSupportRequestedCapability { .. })
    ));
    assert!(matches!(
        layout_admission_registry()
            .admit(scope_mismatch)
            .into_result(),
        Err(LayoutAdmissionDenial::RequestedScopeDoesNotMatchKeyDomain { .. })
    ));
}

#[test]
fn registry_denies_mode_and_mutation_mismatches() {
    let (page_lifecycle, page_domain) = page_scope();
    let verifier = LayoutAdmissionRequest::from_admitted(
        page_lifecycle,
        page_domain,
        LayoutStrategyFamily::BTreeRange,
        LayoutRequestedCapability::point_lookup(),
        ArtifactFamilyAccessLane::HotPath,
    )
    .under_maintenance_mode(IndexMaintenanceMode::VerifierOnly);
    let (wal_lifecycle, wal_domain) = wal_scope();
    let mutation = LayoutAdmissionRequest::from_admitted(
        wal_lifecycle,
        wal_domain,
        LayoutStrategyFamily::BaselineLsmWriteOptimized,
        LayoutRequestedCapability::point_lookup(),
        ArtifactFamilyAccessLane::HotPath,
    )
    .for_mutation_shape(PhysicalMutationShape::PointRewrite);

    assert!(matches!(
        layout_admission_registry().admit(verifier).into_result(),
        Err(LayoutAdmissionDenial::MaintenanceModeIncompatibleWithRequestedLane { .. })
    ));
    assert!(matches!(
        layout_admission_registry().admit(mutation).into_result(),
        Err(LayoutAdmissionDenial::MutationShapeIncompatibleWithStrategy { .. })
    ));
}

#[test]
fn registry_requires_real_coverage_when_exact_materialization_is_requested() {
    let (lifecycle, domain) = page_scope();
    let request = LayoutAdmissionRequest::from_admitted(
        lifecycle,
        domain,
        LayoutStrategyFamily::BTreeRange,
        LayoutRequestedCapability::point_lookup(),
        ArtifactFamilyAccessLane::HotPath,
    )
    .require_exact_readiness();

    let denial = layout_admission_registry().admit(request).unwrap_err();
    assert_eq!(denial, LayoutAdmissionDenial::ExactMaterializationRequired);
    assert_eq!(
        denial.case(),
        crate::strategy::registry::LayoutAdmissionDenialCase::ExactMaterializationRequired
    );
}

#[test]
fn registry_denial_case_inventory_is_exhaustive_and_unique() {
    use std::collections::HashSet;

    let cases = crate::strategy::registry::LayoutAdmissionDenialCase::ALL;
    assert_eq!(cases.len(), 15);
    assert_eq!(cases.into_iter().collect::<HashSet<_>>().len(), cases.len());
}

fn page_scope() -> (
    crate::AdmittedPhysicalArtifactFamily,
    crate::AdmittedPhysicalKeyDomain,
) {
    let (family, domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    (family, domain)
}

fn wal_scope() -> (
    crate::AdmittedPhysicalArtifactFamily,
    crate::AdmittedPhysicalKeyDomain,
) {
    let (family, domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PublicationWalIntent,
        StoreKeyScope::WalCheckpointEnvelope,
        StoreTenantScope::StoreInternal,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedWalRecord,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    (family, domain)
}
