//! Group rosters are semantic claims, even inside authenticated checkpoint frames.
use super::super::{
    correction_courts::{install_definition, reverse},
    lifecycle::*,
    native_checkpoint_tamper::{field, items, optional_field},
    *,
};
use super::{
    history::{reopen, retire_sentinel},
    multi::{create, settle, TwoProgram},
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
type S = Model<20>;
#[derive(Clone, Copy, Debug)]
enum Change {
    Drop,
    Add,
    Repoint,
}
#[test]
fn cold_retirement_owner_roster_is_recomputed_from_preceding_facts() {
    run(false);
}
#[test]
fn cold_restoration_owner_roster_is_recomputed_from_preceding_facts() {
    run(true);
}
fn run(restoration: bool) {
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let first = create(&host, &request, 10, vec![20]);
    let second = create(&host, &request, 11, vec![]);
    create(&host, &request, 40, vec![]);
    settle(&host, &request, 10);
    settle(&host, &request, 11);
    let subject = |receipt: &WorthQueryApplicationCommitReceipt| {
        receipt
            .outputs_of::<CreatedOutputs>()
            .unwrap()
            .entity::<Created<S>>()
            .unwrap()
            .entity_id()
    };
    let subjects = [subject(&first), subject(&second)];
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            co_anchors: vec![],
            scope: 1,
            id: 20,
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"history-owner-set".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    reverse(&request, &receipt, "history-owner-set-restoration");
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    assert!(retire_sentinel(&reopen::<S>(checkpoint.clone())).is_ok());
    for change in [Change::Drop, Change::Add, Change::Repoint] {
        let corrupted = super::super::native_checkpoint_tamper::mutate(&checkpoint, |envelope| {
            let group = optional_field(envelope, "coupled_authored_publication")?;
            let group_bytes = &envelope[group.clone()];
            let sources = &group_bytes[field(group_bytes, "sources")];
            optional_field(
                sources,
                if restoration {
                    "Restoration"
                } else {
                    "Retirement"
                },
            )?;
            let roster = field(group_bytes, "participants");
            let parts = items(&group_bytes[roster.clone()]);
            assert_eq!(parts.len(), 2);
            let roster_bytes = &group_bytes[roster.clone()];
            let first = &roster_bytes[parts[0].clone()];
            let second = &roster_bytes[parts[1].clone()];
            let mut replacement = vec![match change {
                Change::Drop => 0x91,
                Change::Add => 0x93,
                Change::Repoint => 0x92,
            }];
            replacement.extend(first);
            match change {
                Change::Drop => {}
                Change::Repoint => {
                    let origin = field(second, "origin");
                    let publication = field(&second[origin.clone()], "publication");
                    let preserve = field(
                        &second[origin.start + publication.start..origin.start + publication.end],
                        "Preserve",
                    );
                    let base = origin.start + publication.start + preserve.start;
                    let predecessor = field(&second[base..], "predecessor");
                    let mut changed = second.to_vec();
                    changed.splice(
                        base + predecessor.start..base + predecessor.end,
                        rmp_serde::to_vec(&first_birth(first)).unwrap(),
                    );
                    replacement.extend(changed);
                }
                Change::Add => {
                    replacement.extend(second);
                    replacement.extend(extra_participant(first));
                }
            }
            Some((
                group.start + roster.start..group.start + roster.end,
                replacement,
            ))
        });
        let host = reopen::<S>(corrupted);
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        // A restored anchor's next retirement validates its genuine historical birth.
        // No completed Query receipt is presumed to survive a cold reopen.
        let outcome = request
            .advanced()
            .mutate(Retire {
                relation_fault: 0,
                corrupt_restoration: false,
                co_anchors: vec![],
                scope: 1,
                id: 20,
                descendants: vec![],
            })
            .without_source()
            .idempotency(&"corrupt-restored-stage-birth".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let result = match outcome {
            WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => Ok(receipt),
            WorthQueryApplicationMutationOutcome::Commit(
                WorthQueryApplicationUncommitted::Denied(denial),
            ) => Err(denial
                .authored_retirement_error()
                .expect("typed historical birth refusal")
                .clone()),
            _ => panic!("unexpected historical birth outcome: {outcome:?}"),
        };
        let error = result
            .expect_err("historical inverse or birth recomputes its actual recorded owner set");
        assert_eq!(error.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::CorruptPublication), "{change:?}: {error:?}");
        let subject = match change {
            Change::Drop | Change::Repoint => subjects[1],
            Change::Add => subjects[0],
        };
        assert_eq!(
            error.context.affected_records,
            [worth_relational::facade::transactions::RecordRef::Entity(
                subject
            )],
            "{change:?}: {error:?}"
        );
        for id in [10, 11, 40] {
            assert!(request.query(query::Input { id }).execute().is_ok());
        }
    }
}
fn first_birth(part: &[u8]) -> u64 {
    let origin = &part[field(part, "origin")];
    rmp_serde::from_slice(&origin[field(origin, "birth")]).unwrap()
}
fn extra_participant(part: &[u8]) -> Vec<u8> {
    let origin = field(part, "origin");
    let bytes = &part[origin.clone()];
    let meaning = field(bytes, "meaning");
    let family = field(&bytes[meaning.clone()], "family");
    let mut edits = vec![(
        origin.start + meaning.start + family.start..origin.start + meaning.start + family.end,
        rmp_serde::to_vec("foreign.owned.family").unwrap(),
    )];
    let relations = field(bytes, "relations");
    edits.push((
        origin.start + relations.start..origin.start + relations.end,
        vec![0x90],
    ));
    for (field_name, member) in [("members", true), ("outputs", false)] {
        let list = field(bytes, field_name);
        for (index, item) in items(&bytes[list.clone()]).into_iter().enumerate() {
            let offset = list.start + item.start;
            let id = if member {
                field(&bytes[offset..], "id")
            } else {
                0..item.len()
            };
            let slot = field(&bytes[offset + id.start..offset + id.end], "local_slot");
            edits.push((
                origin.start + offset + id.start + slot.start
                    ..origin.start + offset + id.start + slot.end,
                rmp_serde::to_vec(&(900 + index as u64)).unwrap(),
            ));
        }
    }
    edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut changed = part.to_vec();
    for (range, bytes) in edits {
        changed.splice(range, bytes);
    }
    changed
}
