//! Caller mutation of a retained demand cannot redirect recording to another model.
use super::*;
mod drifting;
fn exercise(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_other_model::<drifting::Program>(limits, 1, Some(99));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let document = world::attach(&host);
    let selection = document.selection(&request).unwrap();
    let branch = selection.physical_endpoint().product_branch();
    let drift = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<drifting::Program, drifting::Tree>(
                    &phase,
                    &request,
                    &host,
                    drifting::Demand(std::sync::Arc::clone(&drift)),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    drift.store(true, std::sync::atomic::Ordering::Relaxed);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 5,
        })
        .idempotency_owned(&request, "drift-recording-source".into())
        .unwrap()
        .retain_in_program::<drifting::Program, program::EditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "drifted recording must end");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(_) => panic!("foreign source was recorded"),
            WorthQueryHistoryEditAdvance::Refused {
                edit: owner,
                denial,
            } => {
                let WorthQueryHistoryEditDenial::Outputs(
                    WorthQueryHistoryOutputDenial::Preparation(
                        WorthQueryRequiredOutputPreparationDenial::Demand(
                            WorthQueryApplicationOutputDemandDenial::Demand(cause),
                        ),
                    ),
                ) = &denial
                else {
                    panic!("wrong refusal: {denial:?}")
                };
                assert!(
                    matches!(
                        cause.kind(),
                        WorthQueryOutputDemandDenialKind::ForeignSource
                    ),
                    "{cause:?}"
                );
                assert!(cause.subject().contains("first"), "true family: {cause:?}");
                assert_eq!(
                    host.producer_contacts_on_this_thread_for_test(),
                    contacts,
                    "refuse before any producer contact"
                );
                let document = owner
                    .discard(&request)
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                assert_eq!(document.selection(&request).unwrap(), selection);
                let at = host.request(&principal, &scope).on_branch(branch);
                assert_eq!(
                    at.query(query::model::Read("a".into()))
                        .execute()
                        .unwrap()
                        .rows()[0]
                        .value,
                    1
                );
                assert_eq!(
                    at.query(query::model::Read("b".into()))
                        .execute()
                        .unwrap()
                        .rows()[0]
                        .value,
                    99
                );
                assert!(at
                    .query(query::first_output::Read("first:a".into()))
                    .execute()
                    .is_err());
                assert!(at
                    .query(query::first_output::Read("first:b".into()))
                    .execute()
                    .is_err());
                host.with_host_advancement(|phase| document.close(&phase).unwrap())
                    .unwrap();
                return;
            }
        };
    }
}
#[test]
fn a_drifted_recording_demand_cannot_publish_into_another_document() {
    exercise(false);
}
#[test]
fn worker_only_a_drifted_recording_demand_cannot_publish_into_another_document() {
    exercise(true);
}

mod non_leaf;
mod non_leaf_program;

mod recipe_refusal;

mod ordinary_recipe;
