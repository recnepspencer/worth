

#[allow(unused_imports)]
pub(crate) use {crate::court::reference_context::record_history::growth};



pub(crate) use {crate::court::ordinary_context::record_history::fixture::NativeHistoryCourt};
pub(crate) use {std::num::NonZeroUsize};
pub(crate) use {worth_foundational::facade::AspectKey};
pub(crate) use {worth_relational::facade::history::*, worth_relational::facade::identity::EntityId, worth_relational::facade::transactions::*};

pub(crate) fn limits(entries: usize, visits: usize) -> RelationalRecordHistoryPageLimits {
    RelationalRecordHistoryPageLimits {
        maximum_entries: std::num::NonZeroUsize::new(entries).unwrap(),
        maximum_source_visits: std::num::NonZeroUsize::new(visits).unwrap(),
        maximum_owned_bytes: std::num::NonZeroUsize::new(
            entries * std::mem::size_of::<Option<RelationalRecordHistoryChange>>(),
        )
        .unwrap(),
    }
}

pub(crate) fn assert_retained_equal_update(
    court: &NativeHistoryCourt,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    subjects: &[RelationalRecordHistorySubject],
) {
    let before_basis = court.basis();
    let before = court
        .runtime
        .entity_record_at_observation(&execution, &before_basis.observation(), court.entities[0])
        .unwrap()
        .unwrap();
    court.equal_scalar_edit(execution);
    let basis = court.basis();
    let observation = basis.observation();
    let after = court
        .runtime
        .entity_record_at_observation(&execution, &observation, court.entities[0])
        .unwrap()
        .unwrap();
    assert_eq!(
        before.authoritative_aspect_state, after.authoritative_aspect_state,
        "the real authored edit did not change the retained payload"
    );
    let native = &observation
        .canonical_patch()
        .unwrap()
        .authoritative_record_patches;
    assert_eq!(
        native.len(),
        1,
        "the native owner retained the actual update occurrence"
    );
    assert_eq!(native[0].target, RecordRef::Entity(court.entities[0]));
    assert_eq!(
        native[0].structural_change,
        worth_relational::facade::publication::RecordStructuralChange::Updated
    );
    assert_eq!(
        native[0].authoritative_patch.full_grammar_operation_count(),
        0
    );
    let mut cursor = court.cursor(
        execution,
        &basis,
        subjects,
        &[court.contracts[0].key().clone()],
    );
    let page = cursor
        .next_page(&court.runtime, execution, limits(1, 1))
        .unwrap();
    let event = page
        .entries()
        .next()
        .expect("retained equal update must remain observable");
    assert_eq!(event.record(), &native[0].target);
    assert_eq!(event.commit_id(), observation.commit_id().unwrap());
    assert_eq!(
        event.changed_aspects().count(),
        0,
        "native discarded aspect attribution must not be invented"
    );
}