//! Later stages retain their own correction input and live-owner field selection.
use super::*;
use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome;

#[test]
fn a_later_live_member_edit_retains_its_input_and_reverses_only_its_own_change() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    created(&host, &request, local_image());
    let before = local_snapshot(&request);
    let mut input = selection(&[], None, &[]);
    input.revalue = Some(local_member::Returned {
        key: "part-0".into(),
        value: 77,
    });
    let mut envelope = request
        .edit_envelope()
        .idempotency("source-and-later-member".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    let first = envelope
        .push_stage::<ApplicationNoOutputGraph, _>(edit::source::Intent(edit::Edit {
            key: "source".into(),
            value: 9,
        }))
        .unwrap();
    let later = envelope
        .push_stage::<ApplicationNoOutputGraph, _>(input)
        .unwrap();
    assert_eq!((first.ordinal(), later.ordinal()), (0, 1));
    let outcome = envelope
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryEditEnvelopeOutcome::Committed(done) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(done.stage_count(), 2);
    assert_eq!(done.stage_identity(&later).unwrap().ordinal(), 1);
    let changed = local_snapshot(&request);
    assert_eq!(changed.id, before.id);
    assert_eq!(changed.members[0].0, before.members[0].0);
    assert_eq!(changed.members[0].1.value, 77);
    reverse(
        &request,
        done.stage_receipt(&later).unwrap(),
        "reverse-only-later-member",
    );
    assert_eq!(local_snapshot(&request), before);
    assert_eq!(
        request
            .query(graph_read::source::Read("source".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        9,
        "the later stage's inverse must leave the prefix change intact"
    );
}
