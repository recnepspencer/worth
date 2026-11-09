//! Original fixed delivery remains genuine while an extra family completes recording.
use super::*;
mod fixed;
fn exercise(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program::<fixed::Program>(limits);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let document = world::attach(&host);
    let mut document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<fixed::Program, fixed::Tree>(
                    &phase,
                    &request,
                    &host,
                    first::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
                .with_output_tree::<fixed::Program, program::SecondTree>(
                    &phase,
                    &request,
                    &host,
                    second::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let mut selections = Vec::new();
    let mut outputs = Vec::new();
    for value in [1, 5] {
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let mut edit = document
            .edit(edit::Intent {
                key: "a".into(),
                value,
            })
            .idempotency_owned(&request, format!("fixed-forest-{value}"))
            .unwrap()
            .retain_in_program::<fixed::Program, fixed::Action>(&host)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        for count in 0..512 {
            assert!(count < 511, "fixed forest did not finish");
            edit = match edit.advance(&request) {
                WorthQueryHistoryEditAdvance::Progress(next) => next,
                WorthQueryHistoryEditAdvance::Ready(next) => {
                    edit = next;
                    break;
                }
                WorthQueryHistoryEditAdvance::Refused { denial, .. } => {
                    panic!("fixed forest: {denial:?}")
                }
            };
        }
        let finish = edit
            .finish(&request)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        assert!(matches!(
            &finish.outcome,
            WorthQueryFixedHistoryEditOutcome::Recorded { .. }
        ));
        document = finish.document;
        drop(finish.outcome);
        let selection = document.selection(&request).unwrap();
        let at = host
            .request(&principal, &scope)
            .on_branch(selection.physical_endpoint().product_branch());
        let first = at
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let second = at
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!((first.value, second.value), (value * 10, value + 1000));
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + 2,
            "the already completed fixed root is reused by the configured first tree"
        );
        selections.push(selection);
        outputs.push((first, second));
    }
    assert_eq!(
        (outputs[0].0.native_id, outputs[0].1.native_id),
        (outputs[1].0.native_id, outputs[1].1.native_id)
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut selected = selections[1].clone();
    for (leg, index) in [0, 1, 0].into_iter().enumerate() {
        selected = courts::checkout(
            &document,
            &request,
            selections[index].step().id(),
            selected,
            0x57_ff00 + leg as u128,
        )
        .unwrap_or_else(|cause| panic!("fixed forest selection: {cause:?}"));
        let at = host
            .request(&principal, &scope)
            .on_branch(selected.physical_endpoint().product_branch());
        assert_eq!(
            at.query(query::first_output::Read("first:a".into()))
                .execute()
                .unwrap()
                .rows()[0],
            outputs[index].0
        );
        assert_eq!(
            at.query(query::second_output::Read("second:a".into()))
                .execute()
                .unwrap()
                .rows()[0],
            outputs[index].1
        );
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    }
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn fixed_edit_composes_original_delivery_with_every_configured_tree() {
    exercise(false);
}
#[test]
fn worker_only_fixed_edit_composes_original_delivery_with_every_configured_tree() {
    exercise(true);
}
