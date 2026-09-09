use super::*;
use crate::runtime::intent::UiIntentApplicationFactState;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity};

/// Owner-storage tests use private targets; mounted admission is exercised by
/// the session locality and receipt-succession tests.
fn target(instance: UiMountedInstanceIdentity) -> UiAdmittedValidationAppearanceTarget {
    UiAdmittedValidationAppearanceTarget {
        target: UiValidationAppearanceTarget {
            graph_node: crate::graph::UiGraphNodeIdentity::new(71),
            mounted_instance: instance,
        },
        node_receipt: UiMountedNodeReceiptIdentity::mint_unbound().unwrap(),
    }
}

fn state() -> UiIntentApplicationFactState {
    UiIntentApplicationFactState {
        slots_by_identity: Default::default(),
        facts: Box::default(),
        validation_owner: Some(UiValidationAppearanceOwner::new()),
    }
}

#[test]
fn validation_counter_denials_preserve_facts_revisions_and_identity_allocation() {
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mut state = state();
    state.validation_owner.as_mut().unwrap().revision = u64::MAX;
    let before = state.validation_appearance_snapshot().unwrap();
    assert_eq!(
        state.publish_validation_appearance_fact(
            target(instance),
            None,
            UiValidationAppearanceClass::Invalid
        ),
        Err(UiValidationAppearanceFactDenial::RevisionExhausted)
    );
    assert_eq!(state.validation_appearance_snapshot().unwrap(), before);
    assert_eq!(state.validation_owner.as_ref().unwrap().next_identity, 1);

    state.validation_owner.as_mut().unwrap().revision = 0;
    state.validation_owner.as_mut().unwrap().next_identity = u64::MAX;
    let before = state.validation_appearance_snapshot().unwrap();
    assert_eq!(
        state.publish_validation_appearance_fact(
            target(instance),
            None,
            UiValidationAppearanceClass::Invalid
        ),
        Err(UiValidationAppearanceFactDenial::IdentityExhausted)
    );
    assert_eq!(state.validation_appearance_snapshot().unwrap(), before);
    assert_eq!(
        state.validation_owner.as_ref().unwrap().next_identity,
        u64::MAX
    );

    state.validation_owner.as_mut().unwrap().next_identity = 1;
    state
        .publish_validation_appearance_fact(
            target(instance),
            None,
            UiValidationAppearanceClass::Invalid,
        )
        .unwrap();
    let owner = state.validation_owner.as_mut().unwrap();
    let (graph, mut fact) = *owner.facts.get(&instance).unwrap();
    fact.revision = u64::MAX;
    owner.facts.insert(instance, (graph, fact));
    let before = state.validation_appearance_snapshot().unwrap();
    assert_eq!(
        state.publish_validation_appearance_fact(
            target(instance),
            Some(u64::MAX),
            UiValidationAppearanceClass::Pending
        ),
        Err(UiValidationAppearanceFactDenial::RevisionExhausted)
    );
    assert_eq!(state.validation_appearance_snapshot().unwrap(), before);
    assert_eq!(state.validation_owner.as_ref().unwrap().next_identity, 2);
}

#[test]
fn validation_equal_publication_still_requires_the_exact_predecessor() {
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mut state = state();
    let initial = target(instance);
    let receipt = initial.node_receipt;
    state
        .publish_validation_appearance_fact(initial, None, UiValidationAppearanceClass::Valid)
        .unwrap();
    let before = state.validation_appearance_snapshot().unwrap();
    let mut stale = target(instance);
    stale.node_receipt = receipt;
    assert_eq!(
        state.publish_validation_appearance_fact(stale, None, UiValidationAppearanceClass::Valid),
        Err(UiValidationAppearanceFactDenial::StalePredecessor)
    );
    assert_eq!(state.validation_appearance_snapshot().unwrap(), before);
    let mut current = target(instance);
    current.node_receipt = receipt;
    state
        .publish_validation_appearance_fact(current, Some(1), UiValidationAppearanceClass::Valid)
        .unwrap();
    assert_eq!(state.validation_appearance_snapshot().unwrap(), before);
}

#[test]
fn validation_snapshot_and_target_lookup_do_not_traverse_the_owner_table() {
    for size in [64, 4_096] {
        let mut state = state();
        let mut instances = Vec::new();
        for _ in 0..size {
            let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
            state
                .publish_validation_appearance_fact(
                    target(instance),
                    None,
                    UiValidationAppearanceClass::Valid,
                )
                .unwrap();
            instances.push(instance);
        }
        let graph = crate::graph::UiGraphNodeIdentity::new(71);
        let wrong_graph = crate::graph::UiGraphNodeIdentity::new(72);
        crate::runtime::persistent_index::begin_all_test_observation();
        let before = state.validation_appearance_snapshot().unwrap();
        let clone = before.clone();
        assert_eq!(before, clone);
        assert_eq!(
            crate::runtime::persistent_index::test_work().lookup_probes(),
            0
        );
        assert_eq!(
            crate::runtime::persistent_index::test_work().iterated_entries(),
            0
        );
        assert!(before.fact_basis_for(graph, instances[size / 2]).is_some());
        assert!(before
            .fact_basis_for(wrong_graph, instances[size / 2])
            .is_none());
        assert_eq!(
            before.class_for(wrong_graph, instances[size / 2], None),
            None
        );
        let work = crate::runtime::persistent_index::test_work();
        assert_eq!(work.iterated_entries(), 0);
        assert!(work.lookup_probes() <= 6 * size.ilog2() as usize);

        state.retire_validation_appearance_instance(instances[size / 2]);
        let after = state.validation_appearance_snapshot().unwrap();
        assert_eq!(
            after.changed_instances(&before).as_ref(),
            &[instances[size / 2]]
        );
        assert_eq!(before.fact_count(), size);
        assert_eq!(after.fact_count(), size - 1);
        assert_eq!(
            crate::runtime::persistent_index::test_work().iterated_entries(),
            0
        );
    }
}
