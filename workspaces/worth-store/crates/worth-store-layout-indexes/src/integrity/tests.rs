use crate::integrity::{layout_corruption, LayoutCorruptionView};
use crate::layout_declarations;
use worth_store_contracts::DurableArtifactFamilyId;

pub(super) fn family() -> crate::PhysicalArtifactFamily {
    layout_declarations().seed_family().family()
}

pub(super) fn admitted_family() -> crate::AdmittedPhysicalArtifactFamily {
    crate::strategy::tests_support::root_manifest_scope().0
}

pub(super) fn admitted_family_for_store(
    store_authority_key: &str,
) -> crate::AdmittedPhysicalArtifactFamily {
    crate::strategy::tests_support::admit_strategy_scope_for_store(
        DurableArtifactFamilyId::PhysicalRootManifest,
        worth_store_security::StoreKeyScope::StoreManagedRoot,
        worth_store_security::StoreTenantScope::StoreInternal,
        worth_store_security::StoreAuthenticityRequirement::not_required(),
        worth_store_security::StoreCustodyPosture::InternalStoreCustody,
        store_authority_key,
    )
    .0
}

#[test]
fn corruption_assessment_adapts_physical_and_import_authority() {
    let observation = super::readmission_test_support::authoritative_quarantine_observation(
        "classification-owner",
    );
    let quarantine = layout_corruption().assess_quarantine_observation(
        admitted_family(),
        observation.identity().clone(),
        observation.class(),
    );
    assert!(matches!(
        quarantine.view(),
        LayoutCorruptionView::Quarantined(witness) if witness.family() == family()
    ));

    let terminal = layout_corruption().require_import_readmission(
        admitted_family(),
        super::readmission_test_support::import_witness(family(), "terminal-import"),
    );
    assert!(matches!(
        terminal.view(),
        LayoutCorruptionView::ImportReadmissionRequired(requirement)
            if requirement.family() == family()
    ));
}

#[test]
fn corruption_owner_inventory_equals_ordinary_owner_outputs() {
    use std::collections::BTreeSet;

    let observation = super::readmission_test_support::authoritative_quarantine_observation(
        "classification-matrix",
    );
    let quarantine = layout_corruption().assess_quarantine_observation(
        admitted_family_for_store("store.new.strategy"),
        observation.identity().clone(),
        observation.class(),
    );
    let readmission = layout_corruption()
        .require_observation_bound_recovery_readmission(
            quarantine,
            &super::readmission_test_support::current_authority(
                "store.new.strategy",
                "classification-matrix",
            ),
            super::readmission_test_support::current_security_scope(
                "store.new.strategy",
                "classification-matrix",
            )
            .witnesses(),
        )
        .unwrap();
    let observed = [
        layout_corruption()
            .assess_derived_projection(
                crate::LayoutCorruptionClassification::derived_projection_rebuild_to_parity(),
            )
            .case_id(),
        layout_corruption()
            .assess_quarantine_observation(
                admitted_family(),
                observation.identity().clone(),
                observation.class(),
            )
            .case_id(),
        readmission.case_id(),
        layout_corruption()
            .require_import_readmission(
                admitted_family(),
                super::readmission_test_support::import_witness(family(), "classification-matrix"),
            )
            .case_id(),
    ];

    assert_eq!(
        crate::integrity::corruption_classification_cases().collect::<BTreeSet<_>>(),
        observed.into_iter().collect()
    );
}

pub(super) fn other_family() -> crate::PhysicalArtifactFamily {
    layout_declarations()
        .declaration(DurableArtifactFamilyId::PublicationSnapshotImage)
        .expect("publication snapshot image family should be declared")
        .family()
}
