//! Canonical key DATA, not fabricated publication authority.
use super::*;
use worth_foundational::facade::{DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity};
fn home(seed: u8, slot: u64, branch: &str, incarnation: u64) -> TemporalIdempotencyScope<'_> {
    TemporalIdempotencyScope::Home {
        owner: DurableWorldOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([seed; 16]).unwrap(),
            WorldOwnerSlotIdentity::from_recorded(slot),
        ),
        branch,
        incarnation,
    }
}
#[test]
fn home_temporal_keys_exclude_live_binding_identity_material() {
    let first = runtime_binding_with_live(1, "first-installation-seal", 11);
    let next = runtime_binding_with_live(1, "fresh-installation-seal", 91);
    let key = |binding: &WorthQueryTemporalRuntimeBindingIdentity, scope| {
        prepare_temporal_idempotency(binding, &candidate("input-a"), scope)
            .unwrap()
            .binding()
    };
    assert_eq!(
        key(&first, home(1, 1, "main", 1)).key_identity(),
        key(&next, home(1, 1, "main", 1)).key_identity()
    );
    assert_eq!(
        key(&first, home(1, 1, "main", 1)).intent_identity(),
        key(&next, home(1, 1, "main", 1)).intent_identity()
    );
    assert_ne!(
        key(&first, TemporalIdempotencyScope::ProcessLocal).key_identity(),
        key(&next, TemporalIdempotencyScope::ProcessLocal).key_identity()
    );
}
#[test]
fn home_temporal_keys_distinguish_home_world_branch_lifecycle_and_definition() {
    let binding = runtime_binding(1);
    let key = |scope| {
        prepare_temporal_idempotency(&binding, &candidate("input-a"), scope)
            .unwrap()
            .binding()
    };
    let original = key(home(1, 1, "main", 1));
    for foreign in [
        home(2, 1, "main", 1),
        home(1, 2, "main", 1),
        home(1, 1, "sibling", 1),
        home(1, 1, "main", 2),
    ] {
        assert_ne!(original.key_identity(), key(foreign).key_identity());
    }
    // Vary definition and query independently; a coupled seed could hide an
    // omitted dimension in the production stable-definition builder.
    for changed in [
        runtime_binding_with_node(1, 2, "node-authority", 11),
        runtime_binding_with_node(2, 1, "node-authority", 11),
    ] {
        let different =
            prepare_temporal_idempotency(&changed, &candidate("input-a"), home(1, 1, "main", 1))
                .unwrap()
                .binding();
        assert_ne!(original.key_identity(), different.key_identity());
    }
}
#[test]
fn changed_temporal_input_keeps_home_key_but_changes_intent_identity() {
    let binding = runtime_binding(1);
    let a = prepare_temporal_idempotency(&binding, &candidate("input-a"), home(1, 1, "main", 1))
        .unwrap()
        .binding();
    let b = prepare_temporal_idempotency(&binding, &candidate("input-b"), home(1, 1, "main", 1))
        .unwrap()
        .binding();
    assert_eq!(a.key_identity(), b.key_identity());
    assert_ne!(a.intent_identity(), b.intent_identity());
}
