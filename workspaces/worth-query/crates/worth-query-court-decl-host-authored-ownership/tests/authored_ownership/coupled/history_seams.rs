//! Historical headers cannot adopt an authored anchor or lose a surviving writer.
use super::super::{
    correction_courts::{install_definition, reverse},
    lifecycle::*,
    native_checkpoint_tamper::{field, items, optional_field},
    *,
};
use super::{
    history::reopen,
    multi::{create, settle, TwoProgram},
};
use std::ops::Range;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    identity::EntityId, publication::AuthoredRetirementRefusal, transactions::RecordRef,
};
#[test]
fn cold_coupled_history_refuses_an_authored_anchor_adopted_by_an_earlier_publication() {
    run(false);
}
#[test]
fn cold_coupled_restoration_refuses_an_owner_whose_preceding_publication_has_no_live_member() {
    run(true);
}
fn run(unavailable: bool) {
    type S = Model<6>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let created = create(&host, &request, 10, vec![20]);
    create(&host, &request, 40, vec![]);
    settle(&host, &request, 10);
    let owner = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    let anchor = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<co_roles::CoCreated<S, 0>>()
        .unwrap()
        .unwrap()
        .entity_id();
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
        .idempotency(&"history-seam-retire".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    reverse(&request, &receipt, "history-seam-restore");
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    let changed = super::super::native_checkpoint_tamper::mutate(&checkpoint, |envelope| {
        if unavailable {
            let group = optional_field(envelope, "coupled_authored_publication")?;
            let bytes = &envelope[group.clone()];
            let source = field(bytes, "sources");
            optional_field(&bytes[source], "Retirement")?;
            let parts = field(bytes, "participants");
            let first = items(&bytes[parts.clone()])[0].clone();
            let start = group.start + parts.start + first.start;
            let end = group.start + parts.start + first.end;
            let origin = field(&envelope[start..end], "origin");
            let absolute = start + origin.start..start + origin.end;
            let raw = &envelope[absolute.clone()];
            assert_eq!(source_of(raw), owner);
            let members = field(raw, "members");
            let mut edits = Vec::new();
            for item in items(&raw[members.clone()]) {
                let base = members.start + item.start;
                let member = &raw[base..members.start + item.end];
                for (name, replacement) in [
                    ("posture", rmp_serde::to_vec("Retire").unwrap()),
                    ("before_write", rmp_serde::to_vec(&1u64).unwrap()),
                ] {
                    let field = field(member, name);
                    edits.push((base + field.start..base + field.end, replacement));
                }
            }
            return Some((absolute, replace(raw, edits)));
        }
        let origin = optional_field(envelope, "required_output_origin")?;
        let raw = &envelope[origin.clone()];
        if raw == [0xc0] {
            return None;
        }
        let publication = field(raw, "publication");
        if rmp_serde::from_slice::<String>(&raw[publication])
            .ok()
            .as_deref()
            != Some("Birth")
            || source_of(raw) != owner
        {
            return None;
        }
        let members = field(raw, "members");
        let first = items(&raw[members.clone()])[0].clone();
        let member = field(
            &raw[members.start + first.start..members.start + first.end],
            "id",
        );
        let outputs = field(raw, "outputs");
        let output = items(&raw[outputs.clone()])[0].clone();
        let encoded = rmp_serde::to_vec_named(&anchor).unwrap();
        Some((
            origin,
            replace(
                raw,
                vec![
                    (
                        members.start + first.start + member.start
                            ..members.start + first.start + member.end,
                        encoded.clone(),
                    ),
                    (
                        outputs.start + output.start..outputs.start + output.end,
                        encoded,
                    ),
                ],
            ),
        ))
    });
    let recovered = reopen::<S>(changed);
    let principal = authenticate_operator(recovered.installed_schema(), &scope);
    let request = recovered.runtime().request(&principal, &scope);
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
        .idempotency(&"reject-corrupt-history-seam".to_owned())
        .execute_in_program(&recovered, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}")
    };
    let error = denial
        .authored_retirement_error()
        .expect("original typed historical refusal");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::CorruptPublication)
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(if unavailable { owner } else { anchor })]
    );
    assert!(
        error.detail.contains(if unavailable {
            "coupled restore owner is unavailable"
        } else {
            "coupled anchor belongs to a live publication"
        }),
        "{error:?}"
    );
    for id in [10, 20, 40, 101] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
}
fn source_of(origin: &[u8]) -> EntityId {
    let meaning = field(origin, "meaning");
    let source = field(&origin[meaning.clone()], "source_scope");
    rmp_serde::from_slice(&origin[meaning.start + source.start..meaning.start + source.end])
        .unwrap()
}
fn replace(original: &[u8], mut edits: Vec<(Range<usize>, Vec<u8>)>) -> Vec<u8> {
    edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut result = original.to_vec();
    for (range, bytes) in edits {
        result.splice(range, bytes);
    }
    result
}
