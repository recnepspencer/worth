//! Direct occurrence-update courts for self-referential managed-index Sources.
mod companion;
mod harness;
mod schema;

use companion::ProbeMode;
use harness::Harness;
use schema::{ChildLabelSlot, FieldedSelfSlot};
use std::collections::BTreeSet;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableType;
use worth_relational::facade::{
    identity::{EntityId, PartitionId},
    runtime::ManagedIndexSourcePreparationDenial,
    symbols::ClientKey,
    transactions::{CreatedRelationRef, EntityReference, WorkerIntentBatch},
};

fn projected_self_labels(harness: &Harness, scope: EntityId, row: EntityId) -> Vec<String> {
    let state = harness.state.lock().unwrap();
    state.rows[&scope].child_string_fields_for_test(
        row,
        FieldedSelfSlot::PORTABLE_TYPE_IDENTITY.as_str(),
        ChildLabelSlot::PORTABLE_TYPE_IDENTITY.as_str(),
    )
}

#[test]
fn zero_field_self_referential_endpoint_allows_relation_rewrite_and_tracks_lifecycle() {
    let h = Harness::zero_field();
    let s = h.create_scope("S");
    let a = h.create_node("A", None);
    let b = h.create_node("B", None);
    let t1 = h.create_tag("T1");
    let t2 = h.create_tag("T2");
    let sa = h.link(h.contains_kind, s, a, "S-A");
    let _sb = h.link(h.contains_kind, s, b, "S-B");
    let ba = h.link(h.self_link_kind, b, a, "B-A");
    let at1 = h.link(h.tag_link_kind, a, t1, "A-T1");

    // Case 1: relation-only rewrite of an unrelated relation on A (`ZeroTag`)
    // updates only A; B (`B --ZeroSelf--> A`, 0 fields on A) must not be
    // required or falsely refused with IncompletePreparation.
    let res = h.commit(
        WorkerIntentBatch::new("rewrite-tag-on-a")
            .push(h.unlink_intent(at1))
            .push(h.link_intent(h.tag_link_kind, a, t2, "A-T2")),
    );
    let at2 = res
        .created_relation(&CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: h.tag_link_kind,
            client_key: ClientKey::raw("A-T2"),
            source: EntityReference::Existing(a),
            target: EntityReference::Existing(t2),
        })
        .unwrap();
    {
        let state = h.state.lock().unwrap();
        assert_eq!(state.last_changed_entities, BTreeSet::from([a]));
        assert_eq!(state.last_changed_occurrence_records, Some(Vec::new()));
        assert_eq!(state.last_delta, (vec![a], vec![a]));
    }

    // Lifecycle deletion of A records A in changed_occurrence_record_entities and pulls
    // both A (leaving) and B (losing its ZeroSelf target).
    h.commit(
        WorkerIntentBatch::new("delete-a")
            .push(h.unlink_intent(at2))
            .push(h.unlink_intent(ba))
            .push(h.unlink_intent(sa))
            .push(h.delete_entity_intent(a)),
    );
    let state = h.state.lock().unwrap();
    assert_eq!(state.last_changed_occurrence_records, Some(vec![a]));
    assert_eq!(state.last_changed_entities, BTreeSet::from([a, b]));
    assert!(!state.rows[&s].contains_for_test(a));
    assert!(state.rows[&s].contains_for_test(b));
    let mut expected_before = vec![a, b];
    expected_before.sort_unstable();
    assert_eq!(state.last_delta, (expected_before, vec![b]));
}

#[test]
fn fielded_self_referential_endpoint_separates_relation_rewrite_from_record_write_with_mutation_witnesses(
) {
    let h = Harness::fielded();
    let s = h.create_scope("S");
    let a = h.create_node("A", Some("v1"));
    let b = h.create_node("B", Some("b1"));
    let t1 = h.create_tag("T1");
    let t2 = h.create_tag("T2");
    let _sa = h.link(h.contains_kind, s, a, "S-A");
    let _sb = h.link(h.contains_kind, s, b, "S-B");
    let _ba = h.link(h.self_link_kind, b, a, "B-A");
    let at1 = h.link(h.tag_link_kind, a, t1, "A-T1");
    assert_eq!(projected_self_labels(&h, s, b), vec!["v1".to_string()]);

    // Case 2a + Negative witness #2: relation-only rewrite on A (`FieldedTag`).
    // Dropping A from `changed` MUST refuse IncompletePreparation; full `changed`
    // succeeds with only A in the delta and B untouched.
    h.state.lock().unwrap().probe = Some(ProbeMode::DropFromOccurrences(a));
    h.commit(
        WorkerIntentBatch::new("rewrite-tag-on-a")
            .push(h.unlink_intent(at1))
            .push(h.link_intent(h.tag_link_kind, a, t2, "A-T2")),
    );
    {
        let state = h.state.lock().unwrap();
        assert_eq!(state.last_changed_entities, BTreeSet::from([a]));
        assert_eq!(state.last_changed_occurrence_records, Some(Vec::new()));
        assert!(
            matches!(
                state.last_probe,
                Some(Err(
                    ManagedIndexSourcePreparationDenial::IncompletePreparation
                ))
            ),
            "dropping a changed root row must still refuse IncompletePreparation: {:?}",
            state.last_probe
        );
        assert_eq!(state.last_delta, (vec![a], vec![a]));
    }
    assert_eq!(projected_self_labels(&h, s, b), vec!["v1".to_string()]);

    // Case 2b + Unmasked negative witness #1 (`changed_records.len() <= deps.len()`):
    // scalar write on A.label. Dropping B from BOTH routed rows AND changed_keys
    // bypasses Loop 1 so ONLY the inverse-postings check can catch omitted B.
    h.state.lock().unwrap().probe = Some(ProbeMode::DropRoutedRowAndKey(b));
    h.commit(WorkerIntentBatch::new("update-a-label").push(h.update_label_intent(a, "v2")));
    {
        let state = h.state.lock().unwrap();
        let records = state
            .last_changed_occurrence_records
            .as_ref()
            .expect("ordinary commit carries occurrence record delta");
        assert_eq!(records, &vec![a]);
        assert!(
            records.len() <= state.last_dependency_entity_count,
            "small batch must exercise the changed_records <= dependencies branch"
        );
        assert_eq!(state.last_changed_entities, BTreeSet::from([a, b]));
        assert!(
            matches!(
                state.last_probe,
                Some(Err(ManagedIndexSourcePreparationDenial::IncompletePreparation))
            ),
            "dropping a dependent row on an entity-record write must refuse via inverse postings: {:?}",
            state.last_probe
        );
        let mut expected = vec![a, b];
        expected.sort_unstable();
        assert_eq!(state.last_delta, (expected.clone(), expected));
    }
    assert_eq!(projected_self_labels(&h, s, b), vec!["v2".to_string()]);

    // Missing occurrence-record delta (target-only rebuild) must fail closed with
    // RootUnavailable rather than silently trusting an empty record delta.
    h.state.lock().unwrap().probe = Some(ProbeMode::MissingRecordDelta);
    h.commit(WorkerIntentBatch::new("update-a-label-v3").push(h.update_label_intent(a, "v3")));
    {
        let state = h.state.lock().unwrap();
        assert!(
            matches!(
                state.last_probe,
                Some(Err(ManagedIndexSourcePreparationDenial::RootUnavailable))
            ),
            "unavailable record delta must fail closed with RootUnavailable: {:?}",
            state.last_probe
        );
    }
    assert_eq!(projected_self_labels(&h, s, b), vec!["v3".to_string()]);
}

#[test]
fn fielded_batched_record_write_scans_dependencies_and_catches_omitted_dependent() {
    let h = Harness::fielded();
    let s = h.create_scope("S");
    let a = h.create_node("A", Some("v1"));
    let b = h.create_node("B", Some("b1"));
    let _sa = h.link(h.contains_kind, s, a, "S-A");
    let _sb = h.link(h.contains_kind, s, b, "S-B");
    let _ba = h.link(h.self_link_kind, b, a, "B-A");
    assert_eq!(projected_self_labels(&h, s, b), vec!["v1".to_string()]);

    // Unmasked batched negative witness (`changed_records.len() > deps.len()`):
    // update A together with several unrelated nodes so the dependency-key
    // binary-search scan branch runs and refuses when B is dropped from both
    // routed rows and changed_keys.
    let extras: Vec<_> = (0..5)
        .map(|i| h.create_node(&format!("extra-{i}"), Some("e0")))
        .collect();
    let mut batch =
        WorkerIntentBatch::new("batch-record-write").push(h.update_label_intent(a, "v2"));
    for extra in &extras {
        batch = batch.push(h.update_label_intent(*extra, "e1"));
    }
    h.state.lock().unwrap().probe = Some(ProbeMode::DropRoutedRowAndKey(b));
    h.commit(batch);
    {
        let state = h.state.lock().unwrap();
        let records = state
            .last_changed_occurrence_records
            .as_ref()
            .expect("ordinary batch commit carries occurrence record delta");
        assert_eq!(records.len(), 6);
        assert!(
            records.len() > state.last_dependency_entity_count,
            "large batch must exercise the changed_records > dependencies branch"
        );
        assert!(
            matches!(
                state.last_probe,
                Some(Err(
                    ManagedIndexSourcePreparationDenial::IncompletePreparation
                ))
            ),
            "batched record write dropping dependent B must refuse via inverse postings: {:?}",
            state.last_probe
        );
    }
    assert_eq!(projected_self_labels(&h, s, b), vec!["v2".to_string()]);
}

#[test]
fn two_hop_self_referential_relink_updates_only_direct_owner() {
    let h = Harness::fielded();
    let s = h.create_scope("S");
    let a1 = h.create_node("A1", Some("a1"));
    let a2 = h.create_node("A2", Some("a2"));
    let b = h.create_node("B", Some("b"));
    let c = h.create_node("C", Some("c"));
    for (id, key) in [(a1, "S-A1"), (a2, "S-A2"), (b, "S-B"), (c, "S-C")] {
        let _ = h.link(h.contains_kind, s, id, key);
    }
    let _cb = h.link(h.self_link_kind, c, b, "C-B");
    let ba1 = h.link(h.self_link_kind, b, a1, "B-A1");
    assert_eq!(projected_self_labels(&h, s, b), vec!["a1".to_string()]);
    assert_eq!(projected_self_labels(&h, s, c), vec!["b".to_string()]);

    // Case 3: relinking B --FieldedSelf--> A2 (a 1-hop Source) affects B only;
    // C (2 hops away from A1/A2, projecting only B.label) must not be required.
    h.commit(
        WorkerIntentBatch::new("relink-b")
            .push(h.unlink_intent(ba1))
            .push(h.link_intent(h.self_link_kind, b, a2, "B-A2")),
    );
    {
        let state = h.state.lock().unwrap();
        assert_eq!(state.last_changed_entities, BTreeSet::from([b]));
        assert_eq!(state.last_changed_occurrence_records, Some(Vec::new()));
        assert_eq!(state.last_delta, (vec![b], vec![b]));
    }
    assert_eq!(projected_self_labels(&h, s, b), vec!["a2".to_string()]);
    assert_eq!(projected_self_labels(&h, s, c), vec!["b".to_string()]);
}
