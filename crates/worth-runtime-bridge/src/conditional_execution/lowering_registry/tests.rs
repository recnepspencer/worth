use super::*;

fn key(generation: u64) -> BridgeConditionalLoweringKey {
    BridgeConditionalLoweringKey::successor(
        worth_signal::facade::branch::signal_branch_identity("lowering-registry-test", 1, "main")
            .unwrap(),
        worth_signal::facade::NodeId::new(7, 1),
        generation,
    )
}

#[test]
fn claims_preallocate_and_release_exact_basis_capacity() {
    let registry = Arc::new(RwLock::new(BridgeConditionalLoweringRegistry::default()));
    let first = BridgeConditionalLoweringRegistry::claim(&registry, key(1)).unwrap();
    {
        let state = registry.read().unwrap();
        assert_eq!(state.reserved_exact_basis_slots, 1);
        assert!(state.exact_basis.capacity() > state.exact_basis.len());
    }
    let second = BridgeConditionalLoweringRegistry::claim(&registry, key(2)).unwrap();
    {
        let state = registry.read().unwrap();
        assert_eq!(state.reserved_exact_basis_slots, 2);
        assert!(state.exact_basis.capacity() >= state.exact_basis.len() + 2);
    }

    drop(first);
    assert_eq!(registry.read().unwrap().reserved_exact_basis_slots, 1);
    drop(second);
    let state = registry.read().unwrap();
    assert_eq!(state.reserved_exact_basis_slots, 0);
    assert!(state.slots.is_empty());
    assert_eq!(
        state.exact_basis.alias_owner_count(),
        0,
        "abandoned claim leaves no alias"
    );
}

#[test]
fn abandoning_one_alias_reservation_preserves_another_claims_capacity() {
    let mut index = BridgeExactConditionalBasisIndex::default();
    let owner = key(1);
    index.reserve(&owner, 1).unwrap();
    index.reserve(&owner, 2).unwrap();
    index.release_reservation(&owner);
    assert_eq!(index.alias_owner_count(), 1);
    index.release_reservation(&owner);
    assert_eq!(index.alias_owner_count(), 0);
}
