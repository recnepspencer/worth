use worth_store_physical_integrity::{
    validate_root_manifest, RootManifestIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::super::load::ResidentAdmissionContext;
use super::super::root_manifest::admit_resident_root_manifest;
use super::support::*;
use crate::physical_runtime::ResidentAdmissionCounterCells;

#[test]
fn failed_projection_recheck_is_counted_after_entry() {
    let store = store(78);
    let format = format();
    let bytes = manifest_bytes(15, format);
    let (_pool, _allocation, lease) = loaded_manifest(store, 15, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();
    let scope = scope(store, format, 15, bytes.len());
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&lease);
    let RootManifestIntegrityValidation::Intact(validated) = validate_root_manifest(input, scope).0
    else {
        panic!("the projection fixture must be intact");
    };
    let context = ResidentAdmissionContext::new(lifecycle.observation_state(), &counters);
    let binding = context
        .bind_validated(&lease, scope, validated.into_validation_record())
        .unwrap();

    let denial = context
        .with_owner_projection(binding, || lifecycle.progress_to_record_serving())
        .expect_err("the projection result must not escape its lifecycle generation");

    assert_eq!(
        denial,
        super::super::denial::ResidentIntegrityAdmissionDenial::LifecycleGenerationChanged
    );
    crate::physical_runtime::record_serving::work_semantics::integrity_admission::tests::assert_actual_lifecycle_denial_maps_without_damage(denial);
    crate::physical_runtime::record_serving::assert_actual_lifecycle_manifest_denial_maps_without_damage(denial);
    let observed = counters.snapshot();
    assert_eq!(observed.owner_projection_entries(), 1);
    assert_eq!(observed.refusals_before_owner_entry(), 0);
    assert_eq!(observed.failed_rechecks_after_owner_entry(), 1);
}

#[test]
fn owner_decoder_outcome_is_not_counted_as_an_admission_failure() {
    let store = store(79);
    let format = format();
    let bytes = manifest_bytes(16, format);
    let (_pool, _allocation, lease) = loaded_manifest(store, 16, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();
    let admitted = admit_resident_root_manifest(
        &lease,
        scope(store, format, 16, bytes.len()),
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    )
    .unwrap();

    let outcome = admitted
        .with_owner_decoder(lifecycle.observation_state(), &counters, |_| {
            Err::<(), &'static str>("owner outcome")
        })
        .expect("a current binding must retain the owner decoder outcome");

    assert_eq!(outcome, Err("owner outcome"));
    let observed = counters.snapshot();
    assert_eq!(observed.owner_decoder_entries(), 1);
    assert_eq!(observed.refusals_before_owner_entry(), 0);
    assert_eq!(observed.failed_rechecks_after_owner_entry(), 0);
}
