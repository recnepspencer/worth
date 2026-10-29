pub(crate) use {std::num::NonZeroU64};

pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::PhysicalCheckpointIdentity};
pub(crate) use {worth_store_physical_integrity::validate_checkpoint_binding, worth_store_physical_integrity::validate_checkpoint_binding_compaction, worth_store_physical_integrity::validate_checkpoint_dirty_basis, worth_store_physical_integrity::validate_checkpoint_footer, worth_store_physical_integrity::validate_checkpoint_stream_header, worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation, worth_store_physical_integrity::CheckpointBindingIntegrityValidation, worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation, worth_store_physical_integrity::CheckpointFooterIntegrityValidation, worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityObservationCounters, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
pub(crate) use {crate::checkpoint_validation::support::assert_damage, crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::field_range, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_known, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::other_store, crate::checkpoint_validation::support::reseal_record, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_header};

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Header,
    Dirty,
    Compaction,
    Binding,
}

impl Kind {
    pub(crate) const ALL: [Self; 4] = [Self::Header, Self::Dirty, Self::Compaction, Self::Binding];

    pub(crate) fn bytes(self) -> Vec<u8> {
        match self {
            Self::Header => HEADER.to_vec(),
            Self::Dirty => DIRTY_BASIS.to_vec(),
            Self::Compaction => BINDING_COMPACTION.to_vec(),
            Self::Binding => BINDING.to_vec(),
        }
    }

    pub(crate) fn scope(self) -> PhysicalArtifactScope {
        let identity = identity();
        match self {
            Self::Header => header_scope_known(identity),
            Self::Dirty => dirty_scope(identity),
            Self::Compaction => compaction_scope(identity),
            Self::Binding => binding_scope(identity, BINDING.len() as u64),
        }
    }

    pub(crate) const fn family(self) -> PhysicalIntegrityArtifactFamily {
        match self {
            Self::Header => worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointStreamHeader,
            Self::Dirty => worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis,
            Self::Compaction => worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction,
            Self::Binding => worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBinding,
        }
    }
}

pub(crate) fn rejection(
    kind: Kind,
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes);
    match kind {
        crate::checkpoint_validation::record_matrix::Kind::Header => match validate_checkpoint_stream_header(artifact, scope) {
            (worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            _ => panic!("damaged header unexpectedly validated"),
        },
        crate::checkpoint_validation::record_matrix::Kind::Dirty => match validate_checkpoint_dirty_basis(artifact, scope) {
            (worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            _ => panic!("damaged dirty basis unexpectedly validated"),
        },
        crate::checkpoint_validation::record_matrix::Kind::Compaction => match validate_checkpoint_binding_compaction(artifact, scope) {
            (worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            _ => panic!("damaged compaction unexpectedly validated"),
        },
        crate::checkpoint_validation::record_matrix::Kind::Binding => match validate_checkpoint_binding(artifact, scope) {
            (worth_store_physical_integrity::CheckpointBindingIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            _ => panic!("damaged binding unexpectedly validated"),
        },
    }
}

pub(crate) fn footer_rejection(
    bytes: &[u8],
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    let identity = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
    let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
    let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    match validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        footer_scope(identity),
        basis,
    ) {
        (worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(rejection), counters) => {
            (rejection, counters)
        }
        _ => panic!("damaged footer unexpectedly validated"),
    }
}

pub(crate) fn assert_record_damage(
    kind: Kind,
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = rejection(kind, bytes, scope);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(counters, kind.family(), bytes.len() as u64, Some(cause));
}

pub(crate) fn assert_footer_damage(
    bytes: &[u8],
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let scope = footer_scope(identity());
    let (rejection, counters) = footer_rejection(bytes);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter,
        bytes.len() as u64,
        Some(cause),
    );
}

pub(crate) fn assert_rejected_counters(
    counters: PhysicalIntegrityObservationCounters,
    family: PhysicalIntegrityArtifactFamily,
    bytes: u64,
    cause: Option<PhysicalDamageCause>,
) {
    assert_eq!(counters.family(), family);
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    if let Some(cause) = cause {
        assert_eq!(
            counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(cause)),
            1
        );
    } else {
        assert_eq!(
            counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported),
            1
        );
    }
}

pub(crate) fn store_for_identity() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    identity().store_identity()
}