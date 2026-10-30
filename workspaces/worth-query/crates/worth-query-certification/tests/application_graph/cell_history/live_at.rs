//! Historical selection consumes actual owner observations, never a guessed commit number.
use super::{
    edit::EditCell,
    host::Host,
    query::{CellRow, LiveCells},
};
use crate::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationReadObservation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::WorthQueryApplicationCommitReceipt,
};

pub fn current(host: &Host, feature: u64) -> Vec<CellRow> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .query(LiveCells { feature })
        .execute()
        .expect("current live cells read");
    ordered(result.rows().to_vec())
}
pub fn observe(host: &Host) -> WorthQueryApplicationReadObservation {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let runtime = host.runtime();
    runtime
        .with_application_advancement(&scope, |phase| {
            runtime
                .request(&principal, &scope)
                .retain_read(&phase)
                .expect("the actual feature-step basis is retained")
        })
        .expect("the installed host opens advancement")
}
pub fn at(host: &Host, feature: u64, basis: &WorthQueryApplicationReadObservation) -> Vec<CellRow> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .at(basis)
        .query(LiveCells { feature })
        .execute()
        .expect("retained live cells read");
    ordered(result.rows().to_vec())
}
pub fn at_commit(
    host: &Host,
    feature: u64,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> Vec<CellRow> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let runtime = host.runtime();
    runtime
        .with_application_advancement(&scope, |phase| {
            let result = runtime
                .request(&principal, &scope)
                .at_commit(&phase, receipt, None)
                .expect("the real commit owner selects its feature-step basis")
                .query(LiveCells { feature })
                .execute_in_advancement(&phase)
                .expect("receipt-selected live cells read");
            ordered(result.rows().to_vec())
        })
        .expect("the installed host opens advancement")
}
pub fn edit(
    host: &Host,
    feature: u64,
    cell: u64,
    value: u64,
    key: u64,
) -> WorthQueryApplicationMutationOutcome<super::edit::Denied, super::edit::Edited> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(EditCell {
            feature,
            cell,
            value,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the cell edit reaches the installed program")
}
fn ordered(mut rows: Vec<CellRow>) -> Vec<CellRow> {
    rows.sort_by_key(|row| row.key);
    rows
}
