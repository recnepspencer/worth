mod fixture;

use fixture::NativeHistoryCourt;
use std::num::NonZeroUsize;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::{history::*, identity::EntityId, transactions::*};

fn limits(entries: usize, visits: usize) -> RelationalRecordHistoryPageLimits {
    RelationalRecordHistoryPageLimits {
        maximum_entries: NonZeroUsize::new(entries).unwrap(),
        maximum_source_visits: NonZeroUsize::new(visits).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(
            entries * std::mem::size_of::<Option<RelationalRecordHistoryChange>>(),
        )
        .unwrap(),
    }
}

#[test]
fn canonical_history_pages_stop_inside_a_commit_and_keep_all_grammar_ranks() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let court = NativeHistoryCourt::install(execution);
    let original = court.basis();
    let leases_before = court.runtime.branch_basis_cost_counters();
    let mut subjects: Vec<_> = court
        .entities
        .iter()
        .map(|id| {
            court
                .runtime
                .retain_record_history_subject(execution, &original, RecordRef::Entity(*id))
                .unwrap()
        })
        .collect();
    // Duplicate request targets must not duplicate canonical occurrences.
    subjects.push(subjects[0].clone());
    court.update_all(execution);
    let updated = court.basis();
    let earlier = court
        .runtime
        .select_reachable_commit(
            &updated.observation(),
            original.observation().commit_id().unwrap(),
        )
        .into_outcome()
        .unwrap();
    assert!(matches!(
        court.runtime.record_aspect_history_cursor(
            execution,
            earlier,
            &subjects,
            &[court.contracts[0].key().clone()],
        ),
        Err(RelationalRecordHistoryDenial::NotExactSelectedCommit)
    ));
    let mut keys: Vec<_> = court
        .contracts
        .iter()
        .map(|contract| contract.key().clone())
        .collect();
    keys.sort();
    let mut cursor = court.cursor(execution, &updated, &subjects, &keys);
    let mut observed = Vec::new();
    let mut pages = 0;
    let mut held = None;
    let mut stale = Some(
        cursor
            .prepare_next_page(&court.runtime, execution, limits(2, 2))
            .unwrap(),
    );
    loop {
        let page = cursor
            .next_page(&court.runtime, execution, limits(2, 2))
            .unwrap();
        pages += 1;
        assert!(page.entries().len() <= 2 && page.source_visits() <= 2);
        assert!(page.owned_backing_bytes() <= limits(2, 2).maximum_owned_bytes.get());
        for event in page.entries() {
            assert_eq!(
                event.commit_id(),
                updated.observation().commit_id().unwrap()
            );
            assert_eq!(event.version_id(), updated.observation().version_id());
            assert_eq!(event.changed_aspects().cloned().collect::<Vec<_>>(), keys);
            observed.push(event.record().clone());
        }
        if pages == 1 {
            assert!(matches!(
                cursor.accept_page(stale.take().unwrap()),
                Err(RelationalRecordHistoryDenial::InvalidContinuation)
            ));
            held = page.entries().next().cloned();
            let cancellation = worth_execution::CancellationSource::new();
            let cancelled_host = host.clone().with_cancellation(cancellation.token());
            let controlled = worth_execution::ExecutionRequest::serial(&cancelled_host);
            let prepared = cursor
                .prepare_next_page(&court.runtime, controlled, limits(2, 2))
                .unwrap();
            assert_eq!(prepared.page().entries().len(), 2);
            cancellation.cancel();
            assert!(cursor.accept_page(prepared).is_err());
            let declined = cursor
                .prepare_next_page(&court.runtime, execution, limits(2, 2))
                .unwrap();
            assert_eq!(declined.page().entries().len(), 2);
            drop(declined);
            let foreign_page = cursor
                .prepare_next_page(&court.runtime, execution, limits(2, 2))
                .unwrap();
            let other = court.cursor(execution, &updated, &subjects, &keys);
            assert!(matches!(
                other.accept_page(foreign_page),
                Err(RelationalRecordHistoryDenial::InvalidContinuation)
            ));
            let staged = cursor
                .prepare_next_page(&court.runtime, execution, limits(2, 2))
                .unwrap();
            let (unpublished_page, unpublished_successor) = cursor.accept_page(staged).unwrap();
            assert_eq!(unpublished_page.entries().len(), 2);
            drop((unpublished_page, unpublished_successor));
            let mut insufficient = limits(2, 2);
            insufficient.maximum_owned_bytes = NonZeroUsize::MIN;
            assert!(matches!(
                cursor.next_page(&court.runtime, execution, insufficient),
                Err(RelationalRecordHistoryDenial::PageStorageTooSmall)
            ));
        }
        if page.is_complete() {
            break;
        }
    }
    assert_eq!(
        pages, 3,
        "five real record patches require three two-entry pages"
    );
    let mut expected: Vec<_> = court
        .entities
        .iter()
        .copied()
        .map(RecordRef::Entity)
        .collect();
    expected.sort();
    assert_eq!(observed, expected, "canonical source order survives");
    // Each rank must independently select entries, not merely annotate a row
    // admitted by a different rank. Missing keys cannot counterfeit history.
    for key in &keys {
        let mut selected = court.cursor(execution, &updated, &subjects, std::slice::from_ref(key));
        let page = selected
            .next_page(&court.runtime, execution, limits(5, 5))
            .unwrap();
        assert_eq!(page.entries().len(), 5);
        assert!(page
            .entries()
            .all(|event| event.changed_aspects().eq(std::iter::once(key))));
    }
    let missing = AspectKey::new("not-in-canonical-operations").unwrap();
    let mut filtered = court.cursor(execution, &updated, &subjects, &[missing]);
    let empty = filtered
        .next_page(&court.runtime, execution, limits(1, 1))
        .unwrap();
    assert_eq!(
        (
            empty.entries().len(),
            empty.source_visits(),
            empty.is_complete()
        ),
        (0, 1, false)
    );
    let next = filtered
        .next_page(&court.runtime, execution, limits(1, 1))
        .unwrap();
    assert_eq!((next.source_visits(), next.is_complete()), (1, false));
    assert_retained_equal_update(&court, execution, &subjects);
    drop((next, empty, filtered, cursor, subjects));
    let leases_held = court.runtime.branch_basis_cost_counters();
    assert!(
        leases_held.external_retention_acquires - leases_before.external_retention_acquires
            > leases_held.external_retention_releases - leases_before.external_retention_releases
    );
    assert_eq!(held.as_ref().unwrap().changed_aspects().count(), 3);
    drop(held);
    let leases_after = court.runtime.branch_basis_cost_counters();
    assert_eq!(
        leases_after.external_retention_acquires - leases_before.external_retention_acquires,
        leases_after.external_retention_releases - leases_before.external_retention_releases
    );
}

fn assert_retained_equal_update(
    court: &NativeHistoryCourt,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    subjects: &[RelationalRecordHistorySubject],
) {
    let before_basis = court.basis();
    let before = court
        .runtime
        .entity_record_at_observation(&before_basis.observation(), court.entities[0])
        .unwrap()
        .unwrap();
    court.equal_scalar_edit(execution);
    let basis = court.basis();
    let observation = basis.observation();
    let after = court
        .runtime
        .entity_record_at_observation(&observation, court.entities[0])
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

#[test]
fn history_subjects_keep_exact_generation_tombstones_and_native_owner_custody() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let court = NativeHistoryCourt::install(execution);
    let basis = court.basis();
    let entity = court.entities[4];
    let subject = court
        .runtime
        .retain_record_history_subject(execution, &basis, RecordRef::Entity(entity))
        .unwrap();
    let relation = court
        .runtime
        .retain_record_history_subject(execution, &basis, RecordRef::Relation(court.relation))
        .unwrap();
    let wildcard = EntityId::new(entity.partition_id, entity.local_slot_value(), 0);
    let wrong_generation = EntityId::new(
        entity.partition_id,
        entity.local_slot_value(),
        entity.generation_value() + 1,
    );
    for id in [wildcard, wrong_generation] {
        assert!(matches!(
            court
                .runtime
                .retain_record_history_subject(execution, &basis, RecordRef::Entity(id)),
            Err(RelationalRecordHistoryDenial::SubjectUnavailable)
        ));
    }
    let mut transaction = court
        .runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("delete-history-subject")
                .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                    DeleteEntityIntent { entity_id: entity },
                )))
                .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                    DeleteRelationIntent {
                        relation_id: court.relation,
                    },
                ))),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.commit(&execution, &court.runtime).unwrap();
    let deleted = court.basis();
    let tombstone = court
        .runtime
        .retain_record_history_subject(execution, &deleted, RecordRef::Entity(entity))
        .unwrap();
    assert_eq!(tombstone.record(), subject.record());
    let mut cursor = court.cursor(
        execution,
        &deleted,
        &[subject],
        &[court.contracts[0].key().clone()],
    );
    let page = cursor
        .next_page(&court.runtime, execution, limits(1, 1))
        .unwrap();
    let event = page.entries().next().unwrap();
    assert_eq!(event.record(), tombstone.record());
    assert_eq!(
        event.structural_change(),
        worth_relational::facade::publication::RecordStructuralChange::Deleted
    );
    let mut relation_history = court.cursor(
        execution,
        &deleted,
        &[relation.clone()],
        &[court.contracts[0].key().clone()],
    );
    let unrelated = relation_history
        .next_page(&court.runtime, execution, limits(1, 1))
        .unwrap();
    assert!(unrelated.entries().len() == 0 && !unrelated.is_complete());
    let edge = relation_history
        .next_page(&court.runtime, execution, limits(1, 1))
        .unwrap();
    assert_eq!(edge.entries().next().unwrap().record(), relation.record());
    assert_eq!(
        edge.entries().next().unwrap().structural_change(),
        worth_relational::facade::publication::RecordStructuralChange::Deleted
    );
    let edge_tombstone = court
        .runtime
        .retain_record_history_subject(execution, &deleted, relation.record().clone())
        .unwrap();
    assert_eq!(edge_tombstone.record(), relation.record());
    let foreign = NativeHistoryCourt::install(execution);
    assert!(matches!(
        foreign
            .runtime
            .retain_record_history_subject(execution, &basis, relation.record().clone()),
        Err(RelationalRecordHistoryDenial::ForeignOwner)
    ));
    assert!(matches!(
        cursor.next_page(&foreign.runtime, execution, limits(1, 1)),
        Err(RelationalRecordHistoryDenial::ForeignOwner)
    ));
    let foreign_basis = foreign.basis();
    let foreign_observation = foreign_basis.observation();
    let selected = foreign
        .runtime
        .select_exact_commit(
            &foreign_observation,
            foreign_observation.commit_id().unwrap(),
        )
        .into_outcome()
        .unwrap();
    assert!(matches!(
        court.runtime.record_aspect_history_cursor(
            execution,
            selected,
            &[tombstone],
            &[court.contracts[0].key().clone()]
        ),
        Err(RelationalRecordHistoryDenial::ForeignOwner)
    ));
}
