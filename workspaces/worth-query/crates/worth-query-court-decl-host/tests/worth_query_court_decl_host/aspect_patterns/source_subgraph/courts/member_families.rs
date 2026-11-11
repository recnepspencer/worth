//! One recorded inverse returns zero to many members of two families to their live owner.
use super::super::roster::*;
use super::owned_members::{created, delete, image, local_image, local_snapshot, reverse};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::transactions::RecordRef;
mod owner_fields;
mod reapplication;

fn selection(members: &[&str], operand: Option<&str>, notes: &[&str]) -> local_member::Delete {
    local_member::Delete {
        owner: "roster".into(),
        owner_value: None,
        members: members.iter().map(|key| (*key).into()).collect(),
        operand: operand.map(Into::into),
        notes: notes.iter().map(|key| (*key).into()).collect(),
        revalue: None,
    }
}
/// Notes enter through the ordinary restore action; nothing here is a correction.
fn note(
    host: &world::Host,
    request: &world::Request<'_, '_, '_>,
    notes: &[(&str, u64)],
    marks: &[(&str, u64)],
) {
    let returned = |rows: &[(&str, u64)]| {
        rows.iter()
            .map(|(key, value)| local_member::Returned {
                key: (*key).into(),
                value: *value,
            })
            .collect::<Vec<_>>()
    };
    let result = request
        .advanced()
        .mutate(local_member::Restore {
            owner: "roster".into(),
            owner_value: None,
            members: Vec::new(),
            operand: None,
            notes: returned(notes),
            marks: returned(marks),
            revalue: None,
        })
        .without_source()
        .idempotency(&"note-roster".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
}

#[test]
fn two_deleted_members_of_one_family_return_to_their_live_owner() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let mut graph = image(3);
    for row in &mut graph.members {
        row.next = None;
    }
    created(&host, &request, graph);
    let before = local_snapshot(&request);
    let receipt = delete(
        &host,
        &request,
        selection(&["part-1", "part-2"], None, &[]),
        "delete-two",
    );
    let deleted = local_snapshot(&request);
    assert_eq!(deleted.id, before.id);
    assert_eq!(deleted.members, before.members[..1]);
    reverse(&request, &receipt, "restore-two-members");
    let after = local_snapshot(&request);
    assert_eq!(after.id, before.id);
    assert_eq!(after.head, before.head);
    assert_eq!(after.members.len(), 3);
    assert_eq!(after.members[0], before.members[0]);
    for (old, fresh) in before.members[1..].iter().zip(&after.members[1..]) {
        assert_ne!(old.0, fresh.0);
        assert_eq!(old.1, fresh.1);
    }
}

#[test]
fn one_member_of_each_of_two_families_returns_through_one_inverse() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, local_image());
    note(&host, &request, &[("note-0", 40), ("note-1", 41)], &[]);
    let before = local_snapshot(&request);
    assert_eq!(before.notes.len(), 2);
    let receipt = delete(
        &host,
        &request,
        selection(&["part-0"], Some("part-1"), &["note-1"]),
        "delete-member-and-note",
    );
    let deleted = local_snapshot(&request);
    assert_eq!(deleted.members, before.members[1..]);
    assert_eq!(deleted.notes, before.notes[..1]);
    assert_eq!(deleted.head, None);
    reverse(&request, &receipt, "restore-member-and-note");
    let after = local_snapshot(&request);
    assert_eq!(after.id, before.id);
    assert_eq!(after.head, before.head);
    assert_eq!(after.members.len(), 2);
    assert_ne!(after.members[0].0, before.members[0].0);
    assert_eq!(after.members[0].1, before.members[0].1);
    assert_eq!(after.members[1], before.members[1]);
    assert_eq!(after.notes.len(), 2);
    assert_eq!(after.notes[0], before.notes[0]);
    assert_ne!(after.notes[1].0, before.notes[1].0);
    assert_eq!(
        (&after.notes[1].1, after.notes[1].2),
        (&before.notes[1].1, before.notes[1].2)
    );
}

#[test]
fn a_deleted_note_returns_with_the_mark_it_carried_and_without_one_it_lacked() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, local_image());
    note(
        &host,
        &request,
        &[("note-0", 40), ("note-1", 41)],
        &[("note-1", 7)],
    );
    let before = local_snapshot(&request);
    assert_eq!(
        before.notes.iter().map(|note| note.3).collect::<Vec<_>>(),
        [None, Some(7)]
    );
    // The handler reads each note's mark, so the bare note's absence is an
    // observed fact of the delete and not a missing pre-image.
    let receipt = delete(
        &host,
        &request,
        selection(&[], None, &["note-0", "note-1"]),
        "delete-marked-and-bare",
    );
    assert!(local_snapshot(&request).notes.is_empty());
    reverse(&request, &receipt, "restore-marked-and-bare");
    let after = local_snapshot(&request);
    assert_eq!(after.members, before.members);
    assert_eq!(after.notes.len(), 2);
    for (old, fresh) in before.notes.iter().zip(&after.notes) {
        assert_ne!(old.0, fresh.0);
        assert_eq!((&old.1, old.2, old.3), (&fresh.1, fresh.2, fresh.3));
    }
}

#[test]
fn a_demanded_mark_the_delete_never_read_is_not_taken_as_absent() {
    let host = world::install(world::Fault::UnreadNoteMark, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, local_image());
    note(&host, &request, &[("note-0", 40)], &[]);
    let before = local_snapshot(&request);
    let pinned = request.retain_current_read().unwrap();
    let result = request
        .advanced()
        .mutate(selection(&[], None, &["note-0"]))
        .without_source()
        .idempotency(&"delete-unread-mark".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("{result:?}");
    };
    assert_eq!(
        denial.preimage_retention_denial(),
        Some(WorthQueryPreImageRetentionDenial::MissingDemandedField)
    );
    assert_eq!(
        pinned.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    assert_eq!(before, local_snapshot(&request));
}

#[test]
fn a_changed_value_of_a_member_that_stays_returns_beside_the_deleted_member() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, local_image());
    let before = local_snapshot(&request);
    let mut input = selection(&["part-0"], Some("part-1"), &[]);
    input.revalue = Some(local_member::Returned {
        key: "part-1".into(),
        value: 77,
    });
    let receipt = delete(&host, &request, input, "delete-and-revalue");
    let deleted = local_snapshot(&request);
    assert_eq!(deleted.members.len(), 1);
    assert_eq!(deleted.members[0].0, before.members[1].0);
    assert_ne!(before.members[1].1.value, 77);
    assert_eq!(deleted.members[0].1.value, 77);
    reverse(&request, &receipt, "restore-member-and-value");
    let after = local_snapshot(&request);
    assert_eq!(after.id, before.id);
    assert_eq!(after.head, before.head);
    assert_eq!(after.members.len(), 2);
    assert_ne!(after.members[0].0, before.members[0].0);
    assert_eq!(after.members[0].1, before.members[0].1);
    // The member that stayed keeps its id, and only its retained value returns.
    assert_eq!(after.members[1], before.members[1]);
}

#[test]
fn a_mutation_that_retires_no_member_commits_and_its_inverse_changes_nothing() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, local_image());
    let before = local_snapshot(&request);
    let receipt = delete(&host, &request, selection(&[], None, &[]), "delete-none");
    assert_eq!(before, local_snapshot(&request));
    reverse(&request, &receipt, "restore-none");
    assert_eq!(before, local_snapshot(&request));
}

#[test]
fn an_inverse_cannot_restore_a_member_its_source_did_not_retire() {
    let host = world::install(world::Fault::ExtraRestoredMember, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    created(&host, &request, local_image());
    let receipt = delete(&host, &request, selection(&[], None, &[]), "delete-none");
    let before = local_snapshot(&request);
    let pinned = request.retain_current_read().unwrap();
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let preparation = request
        .corrections()
        .reverse(&selected)
        .idempotency("restore-unretired-member")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = preparation else {
        panic!("inverse did not prepare");
    };
    let result = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("{result:?}");
    };
    let failure = denial
        .relation_preimage_failure()
        .unwrap_or_else(|| panic!("{denial:?}"));
    assert_eq!(
        failure.kind(),
        WorthQueryRelationPreImageDenial::ExtraInverse
    );
    // The refused record is the fresh member no retirement recorded.
    assert!(
        matches!(failure.affected_records(), [RecordRef::Entity(id)] if before.members.iter().all(|(live, _)| live != id))
    );
    assert_eq!(
        pinned.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    assert_eq!(before, local_snapshot(&request));
}
