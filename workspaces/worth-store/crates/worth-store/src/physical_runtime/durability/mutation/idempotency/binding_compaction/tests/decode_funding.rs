//! Binding decode uses actual same-store native backing and releases its temporary peak.

use std::{alloc::Layout, num::NonZeroU64, sync::atomic::AtomicUsize};

use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};
use worth_store_physical_backend::QualifiedRecoveryFilesystemMedia;

use super::super::encoding::{encode_terminal, encode_wal_bound};
use super::*;
use crate::physical_runtime::durability::mutation::idempotency::{
    fate::PersistedPhysicalMutationFate, persisted_binding::tests::fixture as persisted_fixture,
    registry::PhysicalMutationBindingBasis,
};
use crate::physical_runtime::recovery_freshness::{
    checkpoint_binding_decode_peak, decode_checkpoint_evidence,
    StoreRecoveryCheckpointBindingAllocationDenial as Denial,
};
use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    CompletedPhysicalMutationFact, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryFreshnessPort,
    PhysicalRecoveryRejoinResidentDenial, RecordAppendObservation, StoreRecoveryBindingFreshness,
    StoreRecoveryOperationFate,
};
use worth_store_physical_format::PersistedRecordIdentity;

#[test]
fn native_checkpoint_binding_decode_denies_pressure_and_releases_successful_workspace() {
    // Store/key/policy/mutation come from real owners. Reopened group and LSN
    // fields exercise serialization only; this test grants no WAL authority.
    let (mut fixture, binding) = persisted_fixture::binding();
    let key = binding.key().clone();
    let request_fingerprint = binding.fingerprint();
    let request_mutation = binding.mutation();
    let basis =
        PhysicalMutationBindingBasis::new(key.clone(), request_fingerprint, request_mutation);
    assert!(matches!(
        fixture
            .registry
            .admit_unallocated(key.clone(), request_fingerprint, request_mutation,),
        Ok(PhysicalMutationIdempotencyRegistryAdmission::Fresh(_))
    ));
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
        .expect("owner settles the issued request with its actual no-effect fate");
    let no_effect_payload = encode_retained_record(
        fixture.registry.bindings.values().next().unwrap(),
        PhysicalNamespaceDurableCheckpointGeneration::from_namespace_durable_checkpoint(1),
        &[],
    )
    .unwrap();
    let records = vec![
        PersistedRecordIdentity::new([131; 16], 1).unwrap(),
        PersistedRecordIdentity::new([131; 16], 2).unwrap(),
    ]
    .into_boxed_slice();
    let actual_records_bytes = core::mem::size_of_val(&*records) as u64;
    let (arc, _) = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<CompletedPhysicalMutationFact>())
        .unwrap();
    let completed_actual_peak = 2 * binding.bytes().len() as u64
        + 2 * actual_records_bytes
        + arc.pad_to_align().size() as u64;
    let fact = CompletedPhysicalMutationFact::from_persisted_terminal(
        &binding,
        2,
        7,
        records,
        RecordAppendObservation::from_persisted_fields([0; 13]),
    );
    let completed = PersistedPhysicalMutationFate::completed(binding.clone(), fact);
    let cases = [
        (
            no_effect_payload,
            StoreRecoveryOperationFate::ProvenNoEffect,
            0,
        ),
        (
            encode_terminal(&basis, &completed),
            StoreRecoveryOperationFate::AcknowledgedDurable,
            completed_actual_peak,
        ),
        (
            encode_wal_bound(&binding),
            StoreRecoveryOperationFate::Indeterminate,
            2 * binding.bytes().len() as u64,
        ),
    ];
    let context =
        PhysicalBindingDecodingContext::new(fixture.store, fixture.policy, fixture.idempotency);
    fixture.media.close();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(fixture._root.path())
        .expect("same actual persisted Store namespace");
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    assert_eq!(media.store_identity(), fixture.store);
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let original = 16 * 1024;
    let prior = 64;
    let mut coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(1, 4096, 1, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(original)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    let allocations = coordination.certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    for (payload, expected_fate, actual_peak) in cases {
        let workspace = checkpoint_binding_decode_peak(payload.len())
            .expect("closed codec declares a finite temporary peak for its exact payload");
        assert!(workspace > 0 && workspace + prior < original);
        assert!(workspace >= actual_peak,
        "declared peak must cover the actual nested bytes, records, conversion overlap and padded Arc");
        let held_bytes = original - prior - workspace + 1;
        let held = coordination
            .certification_begin_recovery_allocation(NonZeroU64::new(held_bytes).unwrap())
            .expect("actual native competing reservation leaves workspace one byte short");
        let mut window = coordination.begin_source_read_allocation().unwrap();
        window.reserve_total(prior).unwrap();
        let before = allocations.snapshot().for_dimension(dimension);
        let denial = decode_checkpoint_evidence(&mut window, &payload, context, 1).unwrap_err();
        assert_eq!(
            denial,
            Denial::Backing {
                requested: workspace,
                cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                    required: original + 1,
                    admitted: original,
                },
            }
        );
        let after = allocations.snapshot().for_dimension(dimension);
        assert_eq!(after.admissions(), before.admissions());
        assert_eq!(after.admitted_units(), before.admitted_units());
        assert_eq!(after.active_units(), held_bytes + prior);
        assert_eq!(after.denials(), before.denials() + 1);
        assert_eq!(window.charged_bytes(), prior);
        drop(held);

        let before_retry = allocations.snapshot().for_dimension(dimension);
        let evidence = decode_checkpoint_evidence(&mut window, &payload, context, 1)
            .expect("valid same-store compaction payload decodes after actual pressure release")
            .expect("valid codec state produces inline operation evidence");
        assert_eq!(evidence.idempotency_identity(), key.identity().bytes());
        assert_eq!(evidence.mutation_identity(), request_mutation);
        assert_eq!(evidence.request_fingerprint(), request_fingerprint);
        assert_eq!(evidence.fate(), expected_fate);
        assert_eq!(
            evidence.attempt_binding_identity().is_some(),
            expected_fate == StoreRecoveryOperationFate::Indeterminate,
            "only WAL-bound evidence retains the exact persisted-attempt digest"
        );
        assert_eq!(
            evidence.freshness(),
            StoreRecoveryBindingFreshness::Retained
        );
        let after_retry = allocations.snapshot().for_dimension(dimension);
        assert_eq!(after_retry.admissions(), before_retry.admissions() + 1);
        assert_eq!(
            after_retry.admitted_units() - before_retry.admitted_units(),
            workspace
        );
        assert_eq!(
            after_retry.released_units() - before_retry.released_units(),
            workspace
        );
        assert_eq!(after_retry.active_units(), prior);
        assert_eq!(window.charged_bytes(), prior);
        drop(window);
        assert_eq!(
            allocations
                .snapshot()
                .for_dimension(dimension)
                .active_units(),
            0
        );
    }
    drop(coordination);
    drop(media);
    let disposed = allocations.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}
