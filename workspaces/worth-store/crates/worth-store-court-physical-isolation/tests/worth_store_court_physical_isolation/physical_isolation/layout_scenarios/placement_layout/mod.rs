use {worth_store_physical_isolation::MovablePhysicalRefKind, worth_store_physical_isolation::TierMovementReadInterlockPlan, worth_store_physical_isolation::UnsupportedTierMovementClaim, worth_store_physical_isolation::UnsupportedTierMovementRequest};

#[test]
fn unsupported_hardware_placement_claim_is_denied_by_owner_api() {
    let denial = worth_store_physical_isolation::TierMovementReadInterlockPlan::reject_unsupported_tier_movement(
        worth_store_physical_isolation::UnsupportedTierMovementRequest::new(
            worth_store_physical_isolation::MovablePhysicalRefKind::FutureChunk,
            worth_store_physical_isolation::UnsupportedTierMovementClaim::HardwareMediaPlacement,
        ),
    )
    .expect_err("projection shortcut must stay denied");

    assert_eq!(
        denial,
        worth_store_physical_isolation::TierMovementStabilityDenial::UnsupportedTierMovement
    );
}
