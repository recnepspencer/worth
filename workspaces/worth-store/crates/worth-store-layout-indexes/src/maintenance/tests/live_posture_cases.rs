use std::collections::BTreeSet;

use crate::maintenance::{
    live_maintenance_posture, live_maintenance_posture_cases, IndexMaintenanceMode,
    LiveMaintenanceRequest,
};
use crate::strategy::registry::{
    layout_admission_registry, LayoutAdmissionRequest, LayoutRequestedCapability,
};
use crate::strategy::tests_support::{admit_strategy_scope, root_manifest_scope};
use crate::{ArtifactFamilyAccessLane, LayoutStrategyFamily};
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_security::{
    StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
    StoreKeyScope, StoreTenantScope,
};

#[test]
fn live_posture_declares_exactly_the_cases_ordinary_registry_admission_observes() {
    let (family, key_domain) = page_scope();
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let materialization = crate::access_planning()
        .admit_current_catalog_root_materialization(family, &catalog)
        .unwrap();
    let mut observed = BTreeSet::new();

    for (mode, lane) in [
        (
            IndexMaintenanceMode::SynchronousExact,
            ArtifactFamilyAccessLane::HotPath,
        ),
        (
            IndexMaintenanceMode::AsynchronousLagged,
            ArtifactFamilyAccessLane::HotPath,
        ),
        (
            IndexMaintenanceMode::RebuildOnly,
            ArtifactFamilyAccessLane::MaintenancePath,
        ),
        (
            IndexMaintenanceMode::LazyMaterializedOnDemand,
            ArtifactFamilyAccessLane::HotPath,
        ),
        (
            IndexMaintenanceMode::AdvisoryOnly,
            ArtifactFamilyAccessLane::TerminalPath,
        ),
        (
            IndexMaintenanceMode::VerifierOnly,
            ArtifactFamilyAccessLane::TerminalPath,
        ),
        (
            IndexMaintenanceMode::MigrationOnly,
            ArtifactFamilyAccessLane::MaintenancePath,
        ),
    ] {
        let request = LayoutAdmissionRequest::from_admitted(
            family,
            key_domain,
            LayoutStrategyFamily::BTreeRange,
            LayoutRequestedCapability::point_lookup(),
            lane,
        )
        .under_maintenance_mode(mode);
        let snapshot = layout_admission_registry()
            .admit(request)
            .into_result()
            .unwrap();
        observed.insert(
            live_maintenance_posture()
                .classify(LiveMaintenanceRequest::new(&snapshot, &materialization))
                .case_id()
                .as_str(),
        );
    }

    let (other_family, _) = root_manifest_scope();
    let other_materialization = crate::access_planning()
        .admit_current_catalog_root_materialization(other_family, &catalog)
        .unwrap();
    let snapshot = layout_admission_registry()
        .admit(LayoutAdmissionRequest::from_admitted(
            family,
            key_domain,
            LayoutStrategyFamily::BTreeRange,
            LayoutRequestedCapability::point_lookup(),
            ArtifactFamilyAccessLane::HotPath,
        ))
        .into_result()
        .unwrap();
    observed.insert(
        live_maintenance_posture()
            .classify(LiveMaintenanceRequest::new(
                &snapshot,
                &other_materialization,
            ))
            .case_id()
            .as_str(),
    );

    let declared = live_maintenance_posture_cases()
        .map(|case| case.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(observed, declared);
}

fn page_scope() -> (
    crate::AdmittedPhysicalArtifactFamily,
    crate::AdmittedPhysicalKeyDomain,
) {
    admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    )
}
