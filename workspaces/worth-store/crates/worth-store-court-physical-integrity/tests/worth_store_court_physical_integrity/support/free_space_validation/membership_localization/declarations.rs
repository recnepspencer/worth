pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{
    FreeSpaceBlockReference, FreeSpaceKey, FreeSpaceMembershipBlockScopeIdentity,
    PhysicalFreeSpaceMembershipBlock, PhysicalTreeIdentity,
};
pub(crate) use worth_store_physical_integrity::{
    validate_free_space_membership_block, FreeSpaceMembershipBlockIntegrityValidation,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejectionClass, UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::free_space_validation::{
    assert_damage, assert_intact_counters, assert_rejected_counters, format, independent_crc32c,
    membership_reference, membership_scope, range, reseal, store, MEMBERSHIP_COMPLETE_CRC32C,
    MEMBERSHIP_LITERAL, MEMBERSHIP_OFFSET,
};

pub(crate) fn branch_fixture() -> (Vec<u8>, PhysicalArtifactScope, [FreeSpaceBlockReference; 2]) {
    let children = [child_reference(5, 1, 1, 2), child_reference(5, 2, 3, 4)];
    let branch = worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::branch(
        8,
        6,
        3,
        1,
        children.to_vec(),
        2,
    )
    .unwrap();
    let bytes = branch.encode(format());
    let checksum = independent_crc32c(&[&bytes]);
    let scope = worth_store_physical_integrity::PhysicalArtifactScope::free_space_membership_block(
        store(7),
        format(),
        worth_store_physical_format::FreeSpaceMembershipBlockScopeIdentity::new(
            worth_store_physical_format::PhysicalTreeIdentity::new(8).unwrap(),
            branch.reference(checksum),
        ),
        worth_store_physical_integrity::PhysicalByteRange::new(
            MEMBERSHIP_OFFSET,
            bytes.len() as u64,
        )
        .unwrap(),
    );
    (bytes, scope, children)
}

pub(crate) fn one_child_branch_fixture() -> (Vec<u8>, PhysicalArtifactScope) {
    let child = child_reference(5, 1, 1, 2);
    let branch = worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::branch(
        8,
        6,
        3,
        1,
        vec![child],
        2,
    )
    .unwrap();
    let bytes = branch.encode(format());
    let checksum = independent_crc32c(&[&bytes]);
    let scope = worth_store_physical_integrity::PhysicalArtifactScope::free_space_membership_block(
        store(7),
        format(),
        worth_store_physical_format::FreeSpaceMembershipBlockScopeIdentity::new(
            worth_store_physical_format::PhysicalTreeIdentity::new(8).unwrap(),
            branch.reference(checksum),
        ),
        worth_store_physical_integrity::PhysicalByteRange::new(
            MEMBERSHIP_OFFSET,
            bytes.len() as u64,
        )
        .unwrap(),
    );
    (bytes, scope)
}

pub(crate) fn child_reference(
    generation: u64,
    block: u64,
    first: u64,
    last: u64,
) -> FreeSpaceBlockReference {
    worth_store_physical_format::FreeSpaceBlockReference::new(
        generation,
        block,
        0,
        block as u32,
        worth_store_physical_format::FreeSpaceKey::inline(first).unwrap(),
        worth_store_physical_format::FreeSpaceKey::inline(last).unwrap(),
    )
    .unwrap()
}

pub(crate) fn assert_membership_localization(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    offset: u64,
    length: u64,
    field: PhysicalFormatField,
    blast_radius: PhysicalBlastRadius,
) {
    let (
        worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Rejected(
            rejection,
        ),
        counters,
    ) = validate_free_space_membership_block(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    else {
        panic!("damaged membership block validated");
    };
    assert_damage(
        rejection,
        scope,
        cause,
        range(scope, offset, length),
        Some(field),
        blast_radius,
    );
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(cause),
    );
}
