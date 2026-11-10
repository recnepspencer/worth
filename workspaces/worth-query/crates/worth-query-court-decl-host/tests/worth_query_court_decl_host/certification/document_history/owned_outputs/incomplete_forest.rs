//! Configured-tree completeness cannot stand in for the Native route inventory.
use super::*;

fn incomplete(host: &world::Host, fixed: bool) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let courts::Edited {
        document,
        after,
        selection,
        ..
    } = courts::edited(host, &request, false);
    let model = world::model(host, selection.physical_endpoint().product_branch()).native_id;
    let contacts = host.producer_contacts_on_this_thread_for_test();
    if fixed {
        let document = host
            .with_host_advancement(|phase| {
                document
                    .with_outputs::<program::Program, program::FirstTree>(
                        &phase,
                        &request,
                        host,
                        first::Demand("a".into()),
                        WorthQueryOutputDemandControls::default(),
                    )
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
            })
            .unwrap();
        require_refusal(
            host,
            document,
            &request,
            after,
            selection,
            model,
            contacts + 1,
        );
    } else {
        let document = host
            .with_host_advancement(|phase| {
                document
                    .with_output_trees::<program::Program, program::FirstTree>(
                        &phase,
                        &request,
                        host,
                        first::Demand("a".into()),
                        WorthQueryOutputDemandControls::default(),
                    )
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
            })
            .unwrap();
        require_refusal(
            host,
            document,
            &request,
            after,
            selection,
            model,
            contacts + 1,
        );
    }
}

fn require_refusal<O: WorthQueryHistoryOutputsMode<Schema, History>>(
    host: &world::Host,
    document: WorthQueryApplicationHistoryDocument<'_, Schema, History, OrdinaryHistoryPayload, O>,
    request: &family::Request<'_, '_, '_>,
    before: WorthQueryHistoryNodeId,
    selection: WorthQueryHistorySelection,
    model: worth_relational::facade::identity::EntityId,
    contacts: u64,
) {
    let denial = courts::checkout(&document, request, before, selection.clone(), 0x00f0_5703)
        .expect_err("second is a Native-owned route even when it is omitted from configuration");
    let (
        "advance",
        WorthQueryHistoryNavigationAdvanceDenial::Outputs(
            WorthQueryHistoryOutputDenial::GraphQualification(
                WorthQueryHistoryOutputQualificationDenial::Incomplete { model: subject },
            ),
        ),
    ) = denial
    else {
        panic!("{denial:?}")
    };
    assert_eq!(subject, model);
    assert_eq!(document.selection(request).unwrap(), selection);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn a_forest_cannot_omit_a_native_owned_family() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    incomplete(&world::install(), false);
}
#[test]
fn a_fixed_tree_cannot_hide_another_native_owned_family() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    incomplete(&world::install(), true);
}
#[test]
fn worker_only_forest_cannot_omit_a_native_owned_family() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    incomplete(&world::install_worker_only(), false);
}
