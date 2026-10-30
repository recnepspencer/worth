use super::declarations::*;

pub(crate) fn field_range(
    scope: PhysicalArtifactScope,
    offset: u64,
    length: u64,
) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(
        scope.byte_range().offset() + offset,
        length,
    )
    .unwrap()
}

pub(crate) fn assert_root_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
    assert_inner_and_scope_crc(
        bytes,
        scope,
        scope
            .root_routing_block_identity()
            .expect("root mutation uses root scope")
            .reference()
            .checksum(),
    );
}

pub(crate) fn assert_root_checksum_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
    assert_invalid_inner_and_refreshed_scope_crc(
        bytes,
        scope,
        scope
            .root_routing_block_identity()
            .expect("root checksum mutation uses root scope")
            .reference()
            .checksum(),
    );
}

pub(crate) fn assert_segment_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
    assert_inner_and_scope_crc(
        bytes,
        scope,
        scope
            .segment_membership_block_identity()
            .expect("membership mutation uses membership scope")
            .reference()
            .checksum(),
    );
}

pub(crate) fn assert_segment_checksum_mutation_contract(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) {
    assert_invalid_inner_and_refreshed_scope_crc(
        bytes,
        scope,
        scope
            .segment_membership_block_identity()
            .expect("membership checksum mutation uses membership scope")
            .reference()
            .checksum(),
    );
}

pub(crate) fn assert_inner_and_scope_crc(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    scope_crc: u32,
) {
    assert_eq!(scope.byte_range().length(), bytes.len() as u64);
    assert_eq!(
        u32::from_le_bytes(bytes[44..48].try_into().unwrap()),
        independent_crc32c(&[&bytes[..44], &bytes[48..]])
    );
    assert_eq!(scope_crc, independent_crc32c(&[bytes]));
}

pub(crate) fn assert_invalid_inner_and_refreshed_scope_crc(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    scope_crc: u32,
) {
    assert_eq!(scope.byte_range().length(), bytes.len() as u64);
    assert_ne!(
        u32::from_le_bytes(bytes[44..48].try_into().unwrap()),
        independent_crc32c(&[&bytes[..44], &bytes[48..]])
    );
    assert_eq!(scope_crc, independent_crc32c(&[bytes]));
}

pub(crate) fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    assert_eq!(
        rejection,
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(
            worth_store_physical_integrity::PhysicalDamageLocalization::new(
                scope,
                cause,
                range,
                field,
                blast_radius,
            )
        )
    );
}

pub(crate) fn assert_rejected_counters(
    counters: PhysicalIntegrityObservationCounters,
    family: worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily,
    byte_count: u64,
    cause: PhysicalDamageCause,
) {
    assert_eq!(counters.family(), family);
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), byte_count);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(
            worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(cause)
        ),
        1
    );
}

pub(crate) fn validate_root_intact<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedRootRoutingBlock<'media> {
    match validate_root_routing_block(artifact, scope).0 {
        worth_store_physical_integrity::RootRoutingBlockIntegrityValidation::Intact(validated) => {
            validated
        }
        worth_store_physical_integrity::RootRoutingBlockIntegrityValidation::Rejected(
            rejection,
        ) => {
            panic!("clean root-routing block rejected: {rejection:?}")
        }
    }
}

pub(crate) fn root_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    match validate_root_routing_block(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    ) {
        (
            worth_store_physical_integrity::RootRoutingBlockIntegrityValidation::Rejected(
                rejection,
            ),
            counters,
        ) => (rejection, counters),
        (worth_store_physical_integrity::RootRoutingBlockIntegrityValidation::Intact(_), _) => {
            panic!("damaged root-routing block validated")
        }
    }
}

pub(crate) fn validate_segment_intact<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedSegmentMembershipBlock<'media> {
    match validate_segment_membership_block(artifact, scope).0 {
        worth_store_physical_integrity::SegmentMembershipBlockIntegrityValidation::Intact(
            validated,
        ) => validated,
        worth_store_physical_integrity::SegmentMembershipBlockIntegrityValidation::Rejected(
            rejection,
        ) => {
            panic!("clean segment-membership block rejected: {rejection:?}")
        }
    }
}

pub(crate) fn segment_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    match validate_segment_membership_block(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    ) {
        (
            worth_store_physical_integrity::SegmentMembershipBlockIntegrityValidation::Rejected(
                rejection,
            ),
            counters,
        ) => (rejection, counters),
        (
            worth_store_physical_integrity::SegmentMembershipBlockIntegrityValidation::Intact(_),
            _,
        ) => {
            panic!("damaged segment-membership block validated")
        }
    }
}
