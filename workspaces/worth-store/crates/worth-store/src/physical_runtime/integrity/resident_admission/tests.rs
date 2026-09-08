use worth_store_buffer_pool::{PhysicalFrameAccess, PhysicalFrameKey};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, FreeSpaceBlockReference, FreeSpaceKey, RecordAllocationClass,
};

use super::load::ResidentAdmissionContext;
use super::root_manifest::{admit_loaded_root_manifest, admit_resident_root_manifest};
use crate::physical_runtime::{ResidentAdmissionCounterCells, RootProtocolAdmissionDenial};
use worth_store_physical_integrity::{
    validate_root_manifest, RootManifestIntegrityValidation, UntrustedPhysicalArtifact,
};

mod bootstrap_catalog;
mod counter_semantics;
mod free_space_checksum_cost;
mod support;
use support::*;

#[test]
fn exact_same_generation_hit_reuses_record_without_fresh_validation() {
    let store = store(71);
    let format = format();
    let bytes = manifest_bytes(7, format);
    let (pool, allocation, lease) = loaded_manifest(store, 7, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();

    let context = ResidentAdmissionContext::new(lifecycle.observation_state(), &counters);
    let first = admit_resident_root_manifest(&lease, scope(store, format, 7, bytes.len()), context)
        .unwrap();
    first
        .with_owner_decoder(lifecycle.observation_state(), &counters, |view| {
            assert_eq!(view.bytes(), bytes.as_slice())
        })
        .unwrap();
    drop(lease);
    let hit = match pool
        .access_frame(
            &allocation,
            PhysicalFrameKey::new(store, manifest_coordinate(7, bytes.len())),
        )
        .unwrap()
    {
        PhysicalFrameAccess::Hit(lease) => lease,
        _ => panic!("the unchanged frame must remain a resident hit"),
    };
    let second = admit_resident_root_manifest(
        &hit,
        scope(store, format, 7, bytes.len()),
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    )
    .unwrap();
    second
        .with_owner_decoder(lifecycle.observation_state(), &counters, |view| {
            assert_eq!(view.bytes(), bytes.as_slice());
            assert_eq!(view.scope().root_generation(), Some(7));
        })
        .unwrap();

    let observed = counters.snapshot();
    assert_eq!(observed.fresh_validations(), 1);
    assert_eq!(observed.exact_record_reuses(), 1);
    assert_eq!(observed.owner_decoder_entries(), 2);
    assert_eq!(observed.refusals_before_owner_entry(), 0);
    assert_eq!(observed.failed_rechecks_after_owner_entry(), 0);
}

#[test]
fn invalidation_forces_rehash_and_stale_admission_cannot_enter_decoder() {
    let store = store(72);
    let format = format();
    let bytes = manifest_bytes(8, format);
    let (pool, _allocation, lease) = loaded_manifest(store, 8, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();

    let scope = scope(store, format, 8, bytes.len());
    let first = admit_resident_root_manifest(
        &lease,
        scope,
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    )
    .unwrap();
    first
        .with_owner_decoder(lifecycle.observation_state(), &counters, |_| ())
        .unwrap();
    let stale = admit_resident_root_manifest(
        &lease,
        scope,
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    )
    .unwrap();
    let stale_denial = stale
        .with_owner_decoder(lifecycle.observation_state(), &counters, |_| {
            pool.invalidate_integrity_validation_for_runtime_transition();
        })
        .expect_err("post-decoder invalidation must reject the decoder result");
    assert_eq!(stale_denial, RootProtocolAdmissionDenial::ResidentFrame);

    let readmitted = admit_resident_root_manifest(
        &lease,
        scope,
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    )
    .unwrap();
    readmitted
        .with_owner_decoder(lifecycle.observation_state(), &counters, |_| ())
        .unwrap();
    let observed = counters.snapshot();
    assert_eq!(observed.fresh_validations(), 2);
    assert_eq!(observed.exact_record_reuses(), 1);
    assert_eq!(observed.refusals_before_owner_entry(), 0);
    assert_eq!(observed.failed_rechecks_after_owner_entry(), 1);
    assert_eq!(observed.owner_decoder_entries(), 3);
}

#[test]
fn artifact_and_lifecycle_substitution_reject_before_decoder_entry() {
    let store = store(73);
    let format = format();
    let bytes = manifest_bytes(9, format);
    let (_pool, _allocation, lease) = loaded_manifest(store, 9, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();

    let denial = match admit_loaded_root_manifest(
        &lease,
        lifecycle.observation_state(),
        store,
        format,
        10,
        &counters,
    ) {
        Ok(_) => panic!("an artifact-generation substitution must be rejected"),
        Err(denial) => denial,
    };
    assert_eq!(denial, RootProtocolAdmissionDenial::SourceArtifactMismatch);
    let admitted = admit_resident_root_manifest(
        &lease,
        scope(store, format, 9, bytes.len()),
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    )
    .unwrap();
    lifecycle.progress_to_record_serving();
    let lifecycle_denial = admitted
        .with_owner_decoder(lifecycle.observation_state(), &counters, |_| ())
        .expect_err("a lifecycle transition must not open the owner decoder");
    assert_eq!(lifecycle_denial, RootProtocolAdmissionDenial::ResidentFrame);

    let observed = counters.snapshot();
    assert_eq!(observed.fresh_validations(), 1);
    assert_eq!(observed.refusals_before_owner_entry(), 2);
    assert_eq!(observed.failed_rechecks_after_owner_entry(), 0);
    assert_eq!(observed.owner_decoder_entries(), 0);
}

#[test]
fn corrupt_fresh_bytes_reject_before_any_owner_decoder_entry() {
    let store = store(74);
    let format = format();
    let mut bytes = manifest_bytes(10, format);
    let last = bytes.len() - 1;
    bytes[last] ^= 0x80;
    let (_pool, _allocation, lease) = loaded_manifest(store, 10, &bytes);
    let counters = ResidentAdmissionCounterCells::default();

    assert!(admit_loaded_root_manifest(
        &lease,
        lifecycle().observation_state(),
        store,
        format,
        10,
        &counters,
    )
    .is_err());

    let observed = counters.snapshot();
    assert_eq!(observed.fresh_validations(), 1);
    assert_eq!(observed.exact_record_reuses(), 0);
    assert_eq!(observed.refusals_before_owner_entry(), 1);
    assert_eq!(observed.failed_rechecks_after_owner_entry(), 0);
    assert_eq!(observed.owner_decoder_entries(), 0);
}

#[test]
fn exact_binding_rejects_same_scope_record_replacement() {
    let store = store(75);
    let format = format();
    let bytes = manifest_bytes(11, format);
    let (_pool, _allocation, lease) = loaded_manifest(store, 11, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();
    let scope = scope(store, format, 11, bytes.len());
    let admitted = admit_resident_root_manifest(
        &lease,
        scope,
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    )
    .unwrap();

    let key = FreeSpaceKey::new(RecordAllocationClass::InlinePage, 1).unwrap();
    let free_space_root = FreeSpaceBlockReference::new(11, 1, 0, 41, key, key).unwrap();
    let replacement_bytes = DurablePhysicalRootManifest::builder(11, 99, 2, 43)
        .free_space_root(Some(free_space_root))
        .admit()
        .unwrap()
        .encode(format);
    let replacement_input = UntrustedPhysicalArtifact::from_bounded_bytes(&replacement_bytes);
    let RootManifestIntegrityValidation::Intact(replacement) =
        validate_root_manifest(replacement_input, scope).0
    else {
        panic!("replacement fixture must be structurally intact");
    };
    lease
        .commit_integrity_validation(replacement.into_validation_record())
        .unwrap();

    assert_eq!(
        admitted
            .with_owner_decoder(lifecycle.observation_state(), &counters, |_| ())
            .expect_err("a different exact record must not satisfy the captured binding"),
        RootProtocolAdmissionDenial::ResidentFrame,
    );
}

#[test]
fn eviction_reload_gets_new_frame_generation_and_forces_one_new_validation() {
    let store = store(76);
    let format = format();
    let bytes = manifest_bytes(12, format);
    let (pool, allocation, lease) = loaded_manifest(store, 12, &bytes);
    let first_generation = lease.resident_generation();
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();
    let scope = scope(store, format, 12, bytes.len());
    admit_resident_root_manifest(
        &lease,
        scope,
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    )
    .unwrap();
    drop(lease);

    let other_key = PhysicalFrameKey::new(store, manifest_coordinate(13, bytes.len()));
    let PhysicalFrameAccess::Fault(other) = pool.access_frame(&allocation, other_key).unwrap()
    else {
        panic!("the one-frame resident budget must fault the substitute frame");
    };
    drop(
        other
            .load(|target| {
                target.copy_from_slice(&bytes);
                Ok::<(), ()>(())
            })
            .unwrap(),
    );
    let original_key = PhysicalFrameKey::new(store, manifest_coordinate(12, bytes.len()));
    let PhysicalFrameAccess::Fault(reload) = pool.access_frame(&allocation, original_key).unwrap()
    else {
        panic!("the evicted original must reload");
    };
    let reloaded = reload
        .load(|target| {
            target.copy_from_slice(&bytes);
            Ok::<(), ()>(())
        })
        .unwrap();
    assert_ne!(reloaded.resident_generation(), first_generation);
    admit_resident_root_manifest(
        &reloaded,
        scope,
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    )
    .unwrap();

    let observed = counters.snapshot();
    assert_eq!(observed.fresh_validations(), 2);
    assert_eq!(observed.exact_record_reuses(), 0);
}

#[test]
fn terminal_runtime_rejects_before_hashing_an_otherwise_live_frame() {
    let store = store(77);
    let format = format();
    let bytes = manifest_bytes(14, format);
    let (_pool, _allocation, lease) = loaded_manifest(store, 14, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();
    lifecycle.begin_termination();
    lifecycle.finish_closed();

    let denial = match admit_resident_root_manifest(
        &lease,
        scope(store, format, 14, bytes.len()),
        ResidentAdmissionContext::new(lifecycle.observation_state(), &counters),
    ) {
        Ok(_) => panic!("a terminal runtime must reject before validation"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial,
        super::denial::ResidentIntegrityAdmissionDenial::LifecycleGenerationChanged,
    );
    assert_eq!(lease.integrity_validation(), None);
    let observed = counters.snapshot();
    assert_eq!(observed.fresh_validations(), 0);
    assert_eq!(observed.refusals_before_owner_entry(), 1);
    assert_eq!(observed.failed_rechecks_after_owner_entry(), 0);
}
