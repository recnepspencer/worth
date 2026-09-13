use super::readmission_test_support::{
    authoritative_quarantine_observation, current_authority, current_security_scope,
    observation_bound_witness_for_store,
};
use super::tests::{admitted_family, family};
use super::{
    layout_corruption, quarantine_readmission, CorruptionDenial, LayoutReadmissionSource,
    QuarantineReadmissionView,
};

#[test]
fn quarantine_readmission_rejects_equal_observation_from_another_store() {
    let observation = authoritative_quarantine_observation("cross-store-quarantine");
    let required = layout_corruption()
        .require_observation_bound_recovery_readmission(
            layout_corruption().assess_quarantine_observation(
                admitted_family(),
                observation.identity().clone(),
                observation.class(),
            ),
            &current_authority("store.new.strategy", "cross-store-quarantine"),
            current_security_scope("store.new.strategy", "cross-store-quarantine").witnesses(),
        )
        .unwrap()
        .into_quarantine_readmission_requirement()
        .unwrap();
    let foreign = observation_bound_witness_for_store(
        family(),
        &observation,
        "store.new.corruption",
        "cross-store-quarantine",
    );

    assert!(matches!(
        quarantine_readmission().admit(required, foreign).view(),
        QuarantineReadmissionView::Denied(denied)
            if matches!(denied.denial(), CorruptionDenial::FamilyBoundReadmissionWitnessRequired {
                family: actual_family,
                source: LayoutReadmissionSource::QuarantineRecovery,
            } if *actual_family == family())
    ));
}
