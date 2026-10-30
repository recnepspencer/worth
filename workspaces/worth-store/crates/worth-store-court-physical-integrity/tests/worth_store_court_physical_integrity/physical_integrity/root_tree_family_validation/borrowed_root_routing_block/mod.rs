use worth_store_physical_format::RootRoutingCoordinateKey;
use worth_store_physical_integrity::validate_root_routing_block_borrowed;

#[test]
fn borrowed_route_validation_matches_owned_exact_scope_and_placements() {
    let store = crate::support::root_tree_family_validation::store(4);
    let block = crate::support::root_tree_family_validation::root_leaf();
    let bytes = block.encode(crate::support::root_tree_family_validation::format());
    let scope = crate::support::root_tree_family_validation::root_scope(
        store,
        &block,
        &bytes,
        crate::support::root_tree_family_validation::ROOT_BLOCK_OFFSET,
    );
    let mut scratch = Vec::<RootRoutingCoordinateKey>::with_capacity(1);
    let (result, counters) = validate_root_routing_block_borrowed(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        scope,
        &mut scratch,
    )
    .unwrap();
    let worth_store_physical_integrity::BorrowedRootRoutingBlockIntegrityValidation::Intact(
        borrowed,
    ) = result
    else {
        panic!("canonical rooted route was rejected");
    };
    let owned = crate::support::root_tree_family_validation::validate_root_intact(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        scope,
    );
    assert_eq!(borrowed.scope(), scope);
    assert_eq!(
        borrowed.record_format(),
        crate::support::root_tree_family_validation::format()
    );
    assert_eq!(
        borrowed.entries().unwrap().collect::<Vec<_>>().as_slice(),
        owned.entries().unwrap()
    );
    assert_eq!(
        borrowed.into_validation_record(),
        owned.into_validation_record()
    );
    assert_eq!(counters.intact_frames(), 1);
}

#[test]
fn insufficient_coordinate_scratch_is_resource_denial_not_media_rejection() {
    let store = crate::support::root_tree_family_validation::store(4);
    let block = crate::support::root_tree_family_validation::root_leaf();
    let bytes = block.encode(crate::support::root_tree_family_validation::format());
    let scope = crate::support::root_tree_family_validation::root_scope(
        store,
        &block,
        &bytes,
        crate::support::root_tree_family_validation::ROOT_BLOCK_OFFSET,
    );
    let denial = validate_root_routing_block_borrowed(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        scope,
        &mut Vec::<RootRoutingCoordinateKey>::new(),
    )
    .unwrap_err();
    assert_eq!(denial.required, 1);
    assert_eq!(denial.provided, 0);
}

#[test]
fn borrowed_route_accepts_schema_three_with_legacy_primary_metadata() {
    let store = crate::support::root_tree_family_validation::store(4);
    let block = crate::support::root_tree_family_validation::root_leaf();
    let mut bytes = block.encode(crate::support::root_tree_family_validation::format());
    bytes[9] = 3;
    crate::support::root_tree_family_validation::reseal_durable_frame(&mut bytes);
    let scope = crate::support::root_tree_family_validation::root_scope(
        store,
        &block,
        &bytes,
        crate::support::root_tree_family_validation::ROOT_BLOCK_OFFSET,
    );
    let mut scratch = Vec::<RootRoutingCoordinateKey>::with_capacity(1);
    let (result, _) = validate_root_routing_block_borrowed(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        scope,
        &mut scratch,
    )
    .unwrap();
    let worth_store_physical_integrity::BorrowedRootRoutingBlockIntegrityValidation::Intact(
        validated,
    ) = result
    else {
        panic!("accepted schema-three legacy-primary route was rejected");
    };
    assert_eq!(
        validated.entries().unwrap().next().unwrap().content_class(),
        worth_store_physical_format::SelectedRecordContentClass::UnknownLegacy,
    );
    let owned = crate::support::root_tree_family_validation::validate_root_intact(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        scope,
    );
    assert_eq!(
        validated.into_validation_record(),
        owned.into_validation_record()
    );
}
