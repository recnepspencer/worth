use super::*;

#[test]
fn exhausted_definition_frontier_refuses_without_inserting() {
    let owner = IndexingSubsystem::new(&());
    let definition = DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "frontier".into(),
        branch_scoped: false,
        kind: crate::indexes::data::DerivedIndexKind::EntityField {
            field_locator: crate::transactions::data::planned_single_field_locator(
                crate::tests::support::aspect_key("account"),
                crate::tests::support::field_key("name"),
            ),
        },
    };
    let first = owner.register_definition(definition.clone()).unwrap();
    owner.state.write().next_index_id = u64::MAX;
    for _ in 0..2 {
        assert_eq!(
            owner.register_definition(definition.clone()),
            Err(crate::indexes::data::DerivedIndexRegistrationDenial::IdentityExhausted)
        );
    }
    let state = owner.state.read();
    assert_eq!(state.next_index_id, u64::MAX);
    assert_eq!(state.definitions.len(), 1);
    assert_eq!(
        state.definitions.get(&first.index_id).unwrap().as_ref(),
        &first
    );
}

#[test]
fn exhausted_generation_frontier_cannot_wrap_or_issue_again() {
    let owner = IndexingSubsystem::new(&());
    owner.state.write().next_generation_id = u64::MAX - 1;
    assert_eq!(owner.next_generation_id(), Some(u64::MAX - 1));
    assert_eq!(owner.next_generation_id(), None);
    assert_eq!(owner.next_generation_id(), None);
    assert!(owner.reserve_generation_ids(1).is_none());
    assert_eq!(owner.state.read().next_generation_id, u64::MAX);
}
