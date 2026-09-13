use std::num::NonZeroU64;

use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_physical_integrity::{
    validate_checkpoint_binding, validate_checkpoint_binding_compaction,
    validate_checkpoint_dirty_basis, validate_checkpoint_footer, validate_checkpoint_stream_header,
    CheckpointBindingCompactionIntegrityValidation, CheckpointBindingIntegrityValidation,
    CheckpointDirtyBasisIntegrityValidation, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, CheckpointStreamHeaderIntegrityValidation,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityObservationCounters, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

use super::literal_vectors::{BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER};
use super::support::{
    assert_damage, binding_scope, compaction_scope, dirty_scope, field_range, footer_scope,
    header_scope_known, header_scope_staged, identity, other_store, reseal_record,
    validate_binding, validate_compaction, validate_dirty, validate_header,
};

#[derive(Clone, Copy)]
enum Kind {
    Header,
    Dirty,
    Compaction,
    Binding,
}

impl Kind {
    const ALL: [Self; 4] = [Self::Header, Self::Dirty, Self::Compaction, Self::Binding];

    fn bytes(self) -> Vec<u8> {
        match self {
            Self::Header => HEADER.to_vec(),
            Self::Dirty => DIRTY_BASIS.to_vec(),
            Self::Compaction => BINDING_COMPACTION.to_vec(),
            Self::Binding => BINDING.to_vec(),
        }
    }

    fn scope(self) -> PhysicalArtifactScope {
        let identity = identity();
        match self {
            Self::Header => header_scope_known(identity),
            Self::Dirty => dirty_scope(identity),
            Self::Compaction => compaction_scope(identity),
            Self::Binding => binding_scope(identity, BINDING.len() as u64),
        }
    }

    const fn family(self) -> PhysicalIntegrityArtifactFamily {
        match self {
            Self::Header => PhysicalIntegrityArtifactFamily::CheckpointStreamHeader,
            Self::Dirty => PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis,
            Self::Compaction => PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction,
            Self::Binding => PhysicalIntegrityArtifactFamily::CheckpointBinding,
        }
    }
}

#[test]
fn four_pre_footer_kinds_enforce_b_k_l_t_u_and_kind_with_exact_localization() {
    for kind in Kind::ALL {
        let scope = kind.scope();
        let mut covered_flip = kind.bytes();
        covered_flip[16] ^= 1;
        assert_record_damage(
            kind,
            &covered_flip,
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        );

        let mut checksum_flip = kind.bytes();
        let last = checksum_flip.len() - 1;
        checksum_flip[last] ^= 1;
        assert_record_damage(
            kind,
            &checksum_flip,
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        );

        let mut length_lie = kind.bytes();
        length_lie[12..16].copy_from_slice(&1_u32.to_le_bytes());
        reseal_record(&mut length_lie);
        assert_record_damage(
            kind,
            &length_lie,
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            field_range(scope, 12, 4),
            Some(PhysicalFormatField::EncodedLength),
            PhysicalBlastRadius::CanonicalFrame,
        );

        let mut kind_substitution = kind.bytes();
        kind_substitution[9] = if kind_substitution[9] == 5 { 4 } else { 5 };
        reseal_record(&mut kind_substitution);
        assert_record_damage(
            kind,
            &kind_substitution,
            scope,
            PhysicalDamageCause::RecordKindMismatch,
            field_range(scope, 9, 1),
            Some(PhysicalFormatField::CheckpointRecordKind),
            PhysicalBlastRadius::CompleteArtifact,
        );

        let complete = kind.bytes();
        let truncated = &complete[..complete.len() - 1];
        assert_record_damage(
            kind,
            truncated,
            scope,
            PhysicalDamageCause::Truncated,
            PhysicalByteRange::new(scope.byte_range().end_exclusive() - 1, 1).unwrap(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        );

        let mut unsupported = kind.bytes();
        unsupported[8] = 2;
        reseal_record(&mut unsupported);
        let (rejection, counters) = rejection(kind, &unsupported, scope);
        let PhysicalIntegrityRejection::Unsupported(version) = rejection else {
            panic!("unsupported checkpoint schema collapsed into damage");
        };
        assert_eq!(version.scope(), scope);
        assert_eq!(
            version.axis(),
            PhysicalIntegrityVersionAxis::CheckpointRecordSchema
        );
        assert_eq!(version.observed(), 2);
        assert_rejected_counters(counters, kind.family(), unsupported.len() as u64, None);
    }
}

#[test]
fn header_stages_only_sequence_and_known_identity_rejects_substitution() {
    let staged = validate_header(&HEADER, header_scope_staged());
    assert_eq!(staged.checkpoint_identity(), identity());

    let other_sequence =
        PhysicalCheckpointIdentity::new(store_for_identity(), NonZeroU64::new(8).unwrap());
    let scope = header_scope_known(other_sequence);
    let (sequence_rejection, _) = rejection(Kind::Header, &HEADER, scope);
    assert_damage(
        sequence_rejection,
        scope,
        PhysicalDamageCause::SequenceMismatch,
        field_range(scope, 32, 8),
        Some(PhysicalFormatField::CheckpointIdentity),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let other_identity =
        PhysicalCheckpointIdentity::new(other_store(), NonZeroU64::new(7).unwrap());
    let scope = header_scope_known(other_identity);
    let (rejection, _) = rejection(Kind::Header, &HEADER, scope);
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::StoreIdentityMismatch,
        field_range(scope, 16, 16),
        Some(PhysicalFormatField::StoreIdentity),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn footer_enforces_b_k_l_s_t_u_after_typed_prefix_validation() {
    let scope = footer_scope(identity());
    let mut covered_flip = FOOTER;
    covered_flip[40] ^= 1;
    assert_footer_damage(
        &covered_flip,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = FOOTER;
    checksum_flip[155] ^= 1;
    assert_footer_damage(
        &checksum_flip,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut length_lie = FOOTER;
    length_lie[12..16].copy_from_slice(&1_u32.to_le_bytes());
    reseal_record(&mut length_lie);
    assert_footer_damage(
        &length_lie,
        PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 12, 4),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut identity_substitution = FOOTER;
    identity_substitution[32..40].copy_from_slice(&8_u64.to_le_bytes());
    reseal_record(&mut identity_substitution);
    assert_footer_damage(
        &identity_substitution,
        PhysicalDamageCause::SequenceMismatch,
        field_range(scope, 32, 8),
        Some(PhysicalFormatField::CheckpointIdentity),
        PhysicalBlastRadius::CompleteArtifact,
    );

    assert_footer_damage(
        &FOOTER[..155],
        PhysicalDamageCause::Truncated,
        PhysicalByteRange::new(scope.byte_range().end_exclusive() - 1, 1).unwrap(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported = FOOTER;
    unsupported[8] = 2;
    reseal_record(&mut unsupported);
    let (rejection, counters) = footer_rejection(&unsupported);
    let PhysicalIntegrityRejection::Unsupported(version) = rejection else {
        panic!("unsupported footer schema collapsed into damage");
    };
    assert_eq!(
        version.axis(),
        PhysicalIntegrityVersionAxis::CheckpointRecordSchema
    );
    assert_eq!(version.observed(), 2);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::CheckpointFooter,
        unsupported.len() as u64,
        None,
    );
}

fn rejection(
    kind: Kind,
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(bytes);
    match kind {
        Kind::Header => match validate_checkpoint_stream_header(artifact, scope) {
            (CheckpointStreamHeaderIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            _ => panic!("damaged header unexpectedly validated"),
        },
        Kind::Dirty => match validate_checkpoint_dirty_basis(artifact, scope) {
            (CheckpointDirtyBasisIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            _ => panic!("damaged dirty basis unexpectedly validated"),
        },
        Kind::Compaction => match validate_checkpoint_binding_compaction(artifact, scope) {
            (CheckpointBindingCompactionIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            _ => panic!("damaged compaction unexpectedly validated"),
        },
        Kind::Binding => match validate_checkpoint_binding(artifact, scope) {
            (CheckpointBindingIntegrityValidation::Rejected(rejection), counters) => {
                (rejection, counters)
            }
            _ => panic!("damaged binding unexpectedly validated"),
        },
    }
}

fn footer_rejection(
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
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    match validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        footer_scope(identity),
        basis,
    ) {
        (CheckpointFooterIntegrityValidation::Rejected(rejection), counters) => {
            (rejection, counters)
        }
        _ => panic!("damaged footer unexpectedly validated"),
    }
}

fn assert_record_damage(
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

fn assert_footer_damage(
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
        PhysicalIntegrityArtifactFamily::CheckpointFooter,
        bytes.len() as u64,
        Some(cause),
    );
}

fn assert_rejected_counters(
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
            counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(cause)),
            1
        );
    } else {
        assert_eq!(
            counters.rejected_for(PhysicalIntegrityRejectionClass::Unsupported),
            1
        );
    }
}

fn store_for_identity() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    identity().store_identity()
}
