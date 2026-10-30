#[allow(unused_imports)]
use crate::free_space_validation::header_canonicality::*;

#[test]
fn arena_frontier_and_geometry_fail_at_their_exact_fields() {
    let mut missing_frontier = HEADER_LITERAL.to_vec();
    missing_frontier[192..200].fill(0);
    reseal(&mut missing_frontier);
    let scope = header_scope(store(7), independent_crc32c(&[&missing_frontier]));
    let (
        worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection),
        _,
    ) = validate_free_space_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &missing_frontier,
        ),
        scope,
    )
    else {
        panic!("zero arena frontier admitted");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        range(scope, 192, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::AllocationFrontier),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut invalid_alignment = HEADER_LITERAL.to_vec();
    invalid_alignment[208..216].copy_from_slice(&3_u64.to_le_bytes());
    reseal(&mut invalid_alignment);
    let scope = header_scope(store(7), independent_crc32c(&[&invalid_alignment]));
    let (
        worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Rejected(rejection),
        _,
    ) = validate_free_space_header(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &invalid_alignment,
        ),
        scope,
    )
    else {
        panic!("non-power-of-two arena alignment admitted");
    };
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        range(scope, 208, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::Payload),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}
