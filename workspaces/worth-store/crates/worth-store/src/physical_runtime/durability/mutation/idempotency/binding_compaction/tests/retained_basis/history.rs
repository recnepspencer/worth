//! Actual issued-key namespace and codec-only conflicting serialized history.

use crate::physical_runtime::durability::mutation::idempotency::{
    binding_compaction::encoding::{encode_terminal, encode_unsealed},
    fate::PersistedPhysicalMutationFate,
    persisted_binding::tests::fixture as persisted_fixture,
    registry::PhysicalMutationBindingBasis,
    test_support::fingerprint,
};
use crate::physical_runtime::{CompletedPhysicalMutationFact, RecordAppendObservation};
use std::{num::NonZeroU64, path::Path};
use worth_store_physical_format::{
    CheckpointRootBasis, CheckpointWalSourceRange, PersistedRecordIdentity,
    PhysicalCheckpointIdentity, PhysicalCheckpointSource,
};

/// Supplies bytes and persisted namespace inputs, never recovery permission.
pub(in crate::physical_runtime) fn with_conflicting_binding_history<R>(
    run: impl FnOnce(&Path, PhysicalCheckpointSource, [Vec<u8>; 3]) -> R,
) -> R {
    let (fixture, binding) = persisted_fixture::binding();
    let basis = PhysicalMutationBindingBasis::new(
        binding.key().clone(),
        binding.fingerprint(),
        binding.mutation(),
    );
    let alternative = fingerprint(&fixture, 18);
    assert_ne!(alternative, binding.fingerprint());
    let conflicting =
        PhysicalMutationBindingBasis::new(binding.key().clone(), alternative, binding.mutation());
    // The alternate fingerprint is adversarial serialized history for one
    // actual issued key, not registry acceptance or an actual WAL fact.
    let records = vec![
        PersistedRecordIdentity::new([131; 16], 1).unwrap(),
        PersistedRecordIdentity::new([131; 16], 2).unwrap(),
    ]
    .into_boxed_slice();
    let completed = CompletedPhysicalMutationFact::from_persisted_terminal(
        &binding,
        2,
        7,
        records,
        RecordAppendObservation::from_persisted_fields([0; 13]),
    );
    let completed = PersistedPhysicalMutationFate::completed(binding.clone(), completed);
    let payloads = [
        encode_unsealed(&basis),
        encode_unsealed(&conflicting),
        encode_terminal(&basis, &completed),
    ];
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
    run(fixture._root.path(), source, payloads)
}
