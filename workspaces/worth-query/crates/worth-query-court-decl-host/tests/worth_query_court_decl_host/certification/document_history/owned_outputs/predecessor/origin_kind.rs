//! A predecessor whose current publication Native did not produce by a run
//! answers with its exact origin kind, and Query treats that as a data posture
//! (the handler may compute fully), never as a refusal (CP2b slice 4,
//! review condition 7, `output_source_observation/predecessor.rs:25-32`).
use super::courts::{edit_model, reset, settle};
use super::*;
use worth_relational::facade::mvcc::{
    RelationalPredecessorBasisDenial, RelationalPredecessorBasisOrigin,
};

fn historical_selection_predecessor(host: &Host) {
    reset(false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&request, host).unwrap();
    let document = world::attach(host);
    let first = document.current_step(&request).unwrap().id();
    let document = edit_model(document, host, &request, 2);
    let edited = document.selection(&request).unwrap();
    // Navigate back: the output's current publication is now a historical
    // selection, which no producer run made.
    let selected = super::super::courts::checkout(&document, &request, first, edited, 0x5134_0001)
        .unwrap_or_else(|denial| panic!("{denial:?}"));
    let on_selected = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    settle(&on_selected, host).unwrap();
    let before = PROBE.with(|probe| probe.borrow().values.len());
    let document = edit_model(document, host, &request, 3);
    let after = document.selection(&request).unwrap();
    let on_after = host
        .request(&principal, &scope)
        .on_branch(after.physical_endpoint().product_branch());
    settle(&on_after, host)
        .expect("a data posture lets the producer compute fully; nothing refuses");
    let values = PROBE.with(|probe| probe.borrow().values[before..].to_vec());
    assert_eq!(
        values,
        [Err(WorthQueryPredecessorReadDenial::Native(
            RelationalPredecessorBasisDenial::PredecessorBasisUnavailableForOrigin(
                RelationalPredecessorBasisOrigin::HistoricalSelection
            )
        ))],
        "the producer's one predecessor read names the exact origin"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn a_historical_selection_predecessor_is_a_typed_data_posture_not_a_refusal() {
    historical_selection_predecessor(&install(false));
}
#[test]
fn worker_only_historical_selection_predecessor_is_a_typed_data_posture_not_a_refusal() {
    historical_selection_predecessor(&install(true));
}
