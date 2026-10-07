//! Issued-key compaction evidence retains its native owner beyond rebuild scratch.

use super::*;
use crate::physical_runtime::durability::mutation::idempotency::persisted_binding::tests::fixture as persisted_fixture;
use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryFreshnessPort,
    QualifiedRecoveryFilesystemMedia, StoreRecoveryOperationEvidence,
};
use std::num::NonZeroU64;
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};
use worth_store_physical_format::*;
use worth_store_physical_integrity::*;

#[path = "retained_basis/fixture.rs"]
mod fixture;
#[path = "retained_basis/history.rs"]
mod history;
pub(in crate::physical_runtime) use history::with_conflicting_binding_history;

#[test]
fn retained_binding_evidence_keeps_native_charge_after_builder_and_owner_disposal() {
    // Store, policy and issued key are real. The codec fixture's reopened group
    // and LSN are serialization inputs, not a claim of WAL authority.
    let (mut fixture, binding) = persisted_fixture::binding();
    let key = binding.key().clone();
    let fingerprint = binding.fingerprint();
    let mutation = binding.mutation();
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
    let payload = encode_retained_record(
        fixture.registry.bindings.values().next().unwrap(),
        PhysicalNamespaceDurableCheckpointGeneration::from_namespace_durable_checkpoint(1),
        &[],
    )
    .unwrap();
    let source = PhysicalCheckpointSource::secured_concurrent(
        PhysicalCheckpointIdentity::new(fixture.store, NonZeroU64::new(1).unwrap()),
        CheckpointWalSourceRange::new(0, 2).unwrap(),
        CheckpointRootBasis::new(1, 1),
        0,
        fixture.policy.bytes(),
        fixture.idempotency.retention().get().get(),
    )
    .unwrap();
    fixture.media.close();
    let qualified =
        QualifiedRecoveryFilesystemMedia::qualify_existing(fixture._root.path()).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    assert_eq!(media.store_identity(), fixture.store);
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let mut coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(1, 4096, 1, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(16 * 1024)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    let observer = coordination.certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let basis = with_binding(source, &payload, |assembly, binding, exact| {
        let shared = coordination.admit_shared_checkpoint(assembly).unwrap();
        let shared_bytes = shared.owned_heap_bytes().unwrap();
        let mut window = coordination.begin_source_read_allocation().unwrap();
        let prior = 64;
        window.reserve_total(prior).unwrap();
        let mut builder = window.begin_checkpoint_binding_rebuild(&shared, 1).unwrap();
        let prepared = observer.snapshot().for_dimension(dimension);
        let builder_bytes = prepared.active_units() - shared_bytes - prior;
        let evidence_bytes = core::mem::size_of::<StoreRecoveryOperationEvidence>() as u64;
        assert!(
            builder_bytes > evidence_bytes,
            "lookup scratch is distinct from retained evidence"
        );
        builder.consume(binding, exact, &mut window).unwrap();
        let consumed = observer.snapshot().for_dimension(dimension);
        assert_eq!(
            consumed.active_units(),
            prepared.active_units(),
            "decode workspace is already disposed"
        );
        assert_eq!(consumed.admissions(), prepared.admissions() + 1);
        assert!(consumed.released_units() > prepared.released_units());
        let basis = builder.finish().unwrap();
        assert!(basis.matches_checkpoint(&shared.facts()));
        assert_eq!(basis.owned_heap_bytes(), Some(evidence_bytes));
        assert_eq!(basis.charged_bytes(), evidence_bytes);
        let finished = observer.snapshot().for_dimension(dimension);
        assert_eq!(
            finished.active_units(),
            shared_bytes + prior + evidence_bytes
        );
        assert_eq!(
            finished.released_units() - consumed.released_units(),
            builder_bytes - evidence_bytes
        );
        drop(window);
        assert_eq!(
            observer.snapshot().for_dimension(dimension).active_units(),
            shared_bytes + evidence_bytes
        );
        drop(shared);
        assert_eq!(
            observer.snapshot().for_dimension(dimension).active_units(),
            evidence_bytes
        );
        basis
    });
    let retained = basis.charged_bytes();
    drop(coordination);
    drop(media);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        retained
    );
    assert_eq!(basis.owned_heap_bytes(), Some(retained));
    drop(basis);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

fn with_binding<R>(
    source: PhysicalCheckpointSource,
    payload: &[u8],
    run: impl FnOnce(
        ValidatedCheckpointStreamAssembly<'_, '_>,
        &IntegrityValidatedCheckpointBinding<'_>,
        UntrustedPhysicalArtifact<'_>,
    ) -> R,
) -> R {
    fixture::with_bindings(source, &[payload], |assembly, bindings, exact| {
        run(assembly, &bindings[0], exact[0])
    })
}
