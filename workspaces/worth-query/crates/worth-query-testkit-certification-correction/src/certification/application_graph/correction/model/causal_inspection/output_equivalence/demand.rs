//! The declared parity root retains genuine commit and readiness custody.
use super::super::super::{
    authentication::*, host, output_program::OutputProgram, query, schema::*,
};
use super::{calculation, program};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*,
    primary_graph::WorthQueryApplicationCommitReceipt,
};
#[derive(Clone)]
pub struct Demand;
impl WorthQueryApplicationOutputDemand<NumericSchema> for Demand {
    type OutputFamily = calculation::Family;
    fn source_intent(&self) -> query::Read {
        query::Read {
            key: "numeric".to_owned(),
        }
    }
}
pub(super) fn settle(
    application: &host::OutputHost,
) -> (
    WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationReadObservation,
    usize,
) {
    let (receipt, read, contacts, posture) = settle_observation(application);
    assert_eq!(posture, WorthQueryOutputSettlementPosture::Performed);
    (receipt.expect("performed parity receipt"), read, contacts)
}
pub(super) fn settle_observation(
    application: &host::OutputHost,
) -> (
    Option<WorthQueryApplicationCommitReceipt>,
    WorthQueryApplicationReadObservation,
    usize,
    WorthQueryOutputSettlementPosture,
) {
    let scope = request_scope();
    let external = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&external, &scope);
    let mut demand = request
        .demand(Demand)
        .start_in_program::<OutputProgram, program::Root>(application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("the installed synchronous cross-entity producer must settle");
    };
    (
        settled.application_commit_receipt().cloned(),
        settled.observation().retain(),
        settled.producer_contacts_in_this_demand(),
        settled.posture(),
    )
}
