pub(crate) use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
pub(crate) use {std::num::NonZeroU64, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_format::PhysicalCheckpointIdentity, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::UntrustedPhysicalArtifact, crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::field_range, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::store};
pub(crate) use {worth_store_physical_integrity::*};

pub(crate) fn inspect(
    validator: &mut PhysicalIntegrityScrubValidator,
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> PhysicalIntegrityObservationOutcome {
    let (inspection, counters) = validator.inspect(PhysicalIntegrityScrubWindow::new(
        0,
        scope,
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
    ));
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);
    inspection.outcome()
}

pub(crate) fn prefix(validator: &mut PhysicalIntegrityScrubValidator) {
    for (scope, bytes) in [
        (header_scope_staged(), HEADER.as_slice()),
        (dirty_scope(identity()), DIRTY_BASIS.as_slice()),
        (compaction_scope(identity()), BINDING_COMPACTION.as_slice()),
        (
            binding_scope(identity(), BINDING.len() as u64),
            BINDING.as_slice(),
        ),
    ] {
        assert!(matches!(
            inspect(validator, scope, bytes),
            PhysicalIntegrityObservationOutcome::Intact(_)
        ));
    }
}