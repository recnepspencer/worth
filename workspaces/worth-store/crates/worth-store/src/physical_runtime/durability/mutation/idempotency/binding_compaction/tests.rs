use super::decoding::PhysicalBindingCompactionRecordDecodeDenial;
#[path = "tests/codec.rs"]
mod codec;
use super::encoding::COMPACTION_RECORD_DOMAIN;
use super::*;
use crate::physical_runtime::durability::mutation::idempotency::persisted_binding::{
    PhysicalBindingDecodingContext, PhysicalPersistedBindingDecodeDenial,
};
use crate::physical_runtime::durability::mutation::idempotency::registry::{
    PhysicalMutationIdempotencyRegistryAdmission, PhysicalMutationUnresolvedBindingObservation,
};
use crate::physical_runtime::durability::mutation::idempotency::test_support::{
    fingerprint, fixture, mutation,
};
use crate::physical_runtime::{
    PhysicalMutationIdempotencyMaterial, PhysicalMutationProvenNoEffectCause,
};

#[cfg(all(
    feature = "recovery-runtime-owner",
    feature = "certification-test-authority"
))]
#[path = "tests/decode_funding.rs"]
mod decode_funding;

#[cfg(all(
    feature = "recovery-runtime-owner",
    feature = "certification-test-authority"
))]
#[path = "tests/retained_basis.rs"]
mod retained_basis;

#[cfg(all(
    feature = "recovery-runtime-owner",
    feature = "certification-test-authority"
))]
pub(in crate::physical_runtime) use retained_basis::with_conflicting_binding_history;

#[test]
fn compaction_record_decoder_rejects_unknown_state_trailing_bytes_and_foreign_policy() {
    let mut fixture = fixture(4);
    let key = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([91; 32]))
        .unwrap();
    let fingerprint = fingerprint(&fixture, 1);
    let mutation = mutation(&fixture, 1);
    assert!(matches!(
        fixture
            .registry
            .admit_unallocated(key, fingerprint, mutation),
        Ok(PhysicalMutationIdempotencyRegistryAdmission::Fresh(_))
    ));
    let encoded = encode_retained_record(
        fixture.registry.bindings.values().next().unwrap(),
        PhysicalNamespaceDurableCheckpointGeneration::from_namespace_durable_checkpoint(1),
        &[],
    )
    .unwrap();
    let context =
        PhysicalBindingDecodingContext::new(fixture.store, fixture.policy, fixture.idempotency);
    assert!(DecodedPhysicalMutationBindingRecord::decode(&encoded, context).is_ok());

    let mut unknown_state = encoded.clone();
    unknown_state[8 + COMPACTION_RECORD_DOMAIN.len()] = u8::MAX;
    assert!(matches!(
        DecodedPhysicalMutationBindingRecord::decode(&unknown_state, context),
        Err(PhysicalBindingCompactionRecordDecodeDenial::UnknownState)
    ));

    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(matches!(
        DecodedPhysicalMutationBindingRecord::decode(&trailing, context),
        Err(PhysicalBindingCompactionRecordDecodeDenial::Persisted(
            PhysicalPersistedBindingDecodeDenial::TrailingBytes
        ))
    ));

    let foreign = PhysicalBindingDecodingContext::new(
        fixture.store,
        fixture.foreign_policy,
        fixture.idempotency,
    );
    assert!(matches!(
        DecodedPhysicalMutationBindingRecord::decode(&encoded, foreign),
        Err(PhysicalBindingCompactionRecordDecodeDenial::Persisted(
            PhysicalPersistedBindingDecodeDenial::ForeignPolicy
        ))
    ));
    fixture.media.close();
}

#[test]
fn persisted_terminal_fate_round_trips_through_its_closed_decoder_seam() {
    let mut fixture = fixture(4);
    let key = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([92; 32]))
        .unwrap();
    let request_fingerprint = fingerprint(&fixture, 2);
    let request_mutation = mutation(&fixture, 2);
    assert!(fixture
        .registry
        .admit_unallocated(key.clone(), request_fingerprint, request_mutation)
        .is_ok());
    fixture
        .registry
        .cancel_before_group_seal(
            PhysicalMutationUnresolvedBindingObservation::new(
                key.identity(),
                request_fingerprint,
                request_mutation,
            ),
            PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal,
        )
        .unwrap();
    let encoded = encode_retained_record(
        fixture.registry.bindings.values().next().unwrap(),
        PhysicalNamespaceDurableCheckpointGeneration::from_namespace_durable_checkpoint(1),
        &[],
    )
    .unwrap();
    let decoded = DecodedPhysicalMutationBindingRecord::decode(
        &encoded,
        PhysicalBindingDecodingContext::new(fixture.store, fixture.policy, fixture.idempotency),
    )
    .unwrap();
    assert!(matches!(
        decoded,
        DecodedPhysicalMutationBindingRecord::Terminal { basis, fate }
            if basis.key().identity() == key.identity()
                && fate.as_proven_no_effect().unwrap().request_fingerprint() == request_fingerprint
    ));
    fixture.media.close();
}

#[test]
fn live_pre_seal_detail_survives_diagnostic_projection_but_not_persisted_fate() {
    use crate::physical_runtime::{
        ArenaAllocationDenial, PhysicalMutationPreSealAdmissionDetail, RecordAppendDenial,
    };

    let mut fixture = fixture(4);
    let key = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new([93; 32]))
        .unwrap();
    let request_fingerprint = fingerprint(&fixture, 4);
    let request_mutation = mutation(&fixture, 4);
    assert!(fixture
        .registry
        .admit_unallocated(key.clone(), request_fingerprint, request_mutation)
        .is_ok());
    let live = fixture
        .registry
        .cancel_before_group_seal(
            PhysicalMutationUnresolvedBindingObservation::new(
                key.identity(),
                request_fingerprint,
                request_mutation,
            ),
            PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal,
        )
        .unwrap()
        .with_admission_detail(PhysicalMutationPreSealAdmissionDetail::RecordPlanning(
            RecordAppendDenial::ArenaAllocationUnavailable(ArenaAllocationDenial::RangeBudget {
                required: 16,
                maximum: 15,
            }),
        ));
    let diagnostic = live.diagnostic_evidence();
    assert_eq!(
        diagnostic.admission_detail(),
        Some(&PhysicalMutationPreSealAdmissionDetail::RecordPlanning(
            RecordAppendDenial::ArenaAllocationUnavailable(ArenaAllocationDenial::RangeBudget {
                required: 16,
                maximum: 15,
            })
        ))
    );
    assert_eq!(
        diagnostic.cause(),
        PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
    );
    let persisted_fate = crate::physical_runtime::durability::mutation::idempotency::fate::PersistedPhysicalMutationFate::proven_no_effect(
        live.clone(),
    );
    let plain = fixture
        .registry
        .bindings
        .values()
        .next()
        .and_then(|state| match state {
            PhysicalMutationIdempotencyBindingState::Terminal { fate, .. } => {
                fate.as_proven_no_effect()
            }
            _ => None,
        })
        .expect("the owner stores the broad terminal fate");
    let mut with_detail_bytes = Vec::new();
    let mut plain_bytes = Vec::new();
    persisted_fate.encode(&mut with_detail_bytes);
    crate::physical_runtime::durability::mutation::idempotency::fate::PersistedPhysicalMutationFate::proven_no_effect(plain)
        .encode(&mut plain_bytes);
    assert_eq!(with_detail_bytes, plain_bytes);
    assert!(format!(
        "{:?}",
        crate::physical_runtime::BlobAppendFailure::ProvenNoEffect(live)
    )
    .contains("RangeBudget { required: 16, maximum: 15 }"));
    let encoded = encode_retained_record(
        fixture.registry.bindings.values().next().unwrap(),
        PhysicalNamespaceDurableCheckpointGeneration::from_namespace_durable_checkpoint(1),
        &[],
    )
    .unwrap();
    let decoded = DecodedPhysicalMutationBindingRecord::decode(
        &encoded,
        PhysicalBindingDecodingContext::new(fixture.store, fixture.policy, fixture.idempotency),
    )
    .unwrap();
    let DecodedPhysicalMutationBindingRecord::Terminal { fate, .. } = decoded else {
        panic!("a settled pre-seal denial must remain a terminal fate")
    };
    let persisted = fate.as_proven_no_effect().unwrap();
    assert_eq!(persisted.cause().encoding_code(), 4);
    assert_eq!(persisted.admission_detail(), None);
    fixture.media.close();
}

#[test]
fn selected_manifest_pins_only_its_exact_expired_negative_binding_across_reopen() {
    let mut fixture = fixture(4);
    let store = fixture.store.bytes();
    let attempt = [41; 16];
    let key = fixture
        .registry
        .issue_key(PhysicalMutationIdempotencyMaterial::new(
            manifest_pin::drop_material(store, attempt),
        ))
        .unwrap();
    let fingerprint = fingerprint(&fixture, 3);
    let mutation = mutation(&fixture, 3);
    assert!(matches!(
        fixture
            .registry
            .admit_unallocated(key.clone(), fingerprint, mutation),
        Ok(PhysicalMutationIdempotencyRegistryAdmission::Fresh(_))
    ));
    fixture
        .registry
        .cancel_before_group_seal(
            PhysicalMutationUnresolvedBindingObservation::new(
                key.identity(),
                fingerprint,
                mutation,
            ),
            PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal,
        )
        .unwrap();
    let state = fixture.registry.bindings.get_mut(&key.identity()).unwrap();
    let PhysicalMutationIdempotencyBindingState::Terminal { last_compacted, .. } = state else {
        panic!("the original drop must have a negative terminal binding")
    };
    *last_compacted =
        Some(PhysicalNamespaceDurableCheckpointGeneration::from_namespace_durable_checkpoint(3));
    fixture
        .registry
        .set_namespace_durable_generation_for_test(3);
    let selected = crate::physical_runtime::record_serving::SelectedBlobManifestPins::one_for_test(
        store, attempt,
    );
    let checkpoint =
        PhysicalCheckpointIdentity::new(fixture.store, std::num::NonZeroU64::new(1).unwrap());
    let retained = fixture
        .registry
        .prepare_binding_compaction(checkpoint, 1, selected, 4096)
        .unwrap();
    assert_eq!(retained.authority.binding_count(), 1);
    let mut records = Vec::new();
    retained
        .for_each_record(&fixture.registry, |record| {
            records.push(record.to_vec());
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
    let decoded = DecodedPhysicalMutationBindingRecord::decode(
        &records[0],
        PhysicalBindingDecodingContext::new(fixture.store, fixture.policy, fixture.idempotency),
    )
    .unwrap();
    assert!(matches!(
        decoded,
        DecodedPhysicalMutationBindingRecord::Terminal { basis, fate }
            if basis.key().identity() == key.identity()
                && fate.as_proven_no_effect().unwrap().request_fingerprint() == fingerprint
    ));
    let proof = fixture.registry.reconcile_original_drop_no_effect(
        store,
        attempt,
        key.identity().bytes(),
        fingerprint.bytes(),
    );
    assert!(proof.is_some());
    assert!(fixture
        .registry
        .reconcile_original_drop_no_effect(
            store,
            [42; 16],
            key.identity().bytes(),
            fingerprint.bytes(),
        )
        .is_none());
    assert!(fixture
        .registry
        .reconcile_original_drop_no_effect(store, attempt, key.identity().bytes(), [99; 32],)
        .is_none());
    let released = fixture
        .registry
        .prepare_binding_compaction(
            checkpoint,
            1,
            crate::physical_runtime::record_serving::SelectedBlobManifestPins::empty_for_test(),
            4096,
        )
        .unwrap();
    assert_eq!(released.authority.binding_count(), 0);
    let wrong_attempt =
        crate::physical_runtime::record_serving::SelectedBlobManifestPins::one_for_test(
            store, [42; 16],
        );
    assert_eq!(
        fixture
            .registry
            .prepare_binding_compaction(checkpoint, 1, wrong_attempt, 4096)
            .unwrap()
            .authority
            .binding_count(),
        0
    );
    assert!(matches!(
        fixture.registry.prepare_binding_compaction(
            checkpoint,
            1,
            crate::physical_runtime::record_serving::SelectedBlobManifestPins::one_for_test(
                store, attempt
            ),
            1,
        ),
        Err(PhysicalMutationBindingCompactionDenial::BindingBytesExceeded)
    ));
    fixture.media.close();
}
