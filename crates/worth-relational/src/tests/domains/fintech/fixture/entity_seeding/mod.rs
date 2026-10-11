mod bulk_create;
mod ledger_entities;
mod market_entities;
mod risk_entities;

use std::collections::BTreeMap;

use crate::facade::identity::EntityId;
use crate::facade::runtime::RelationalRuntime;

use self::ledger_entities::{
    seed_accounts, seed_audit_records, seed_books, seed_cash_events, seed_counterparties,
    seed_desks, seed_settlements, seed_trades,
};
use self::market_entities::{seed_instruments, seed_market_points};
use self::risk_entities::{seed_breaches, seed_limits, seed_risk_views};
use super::seed_catalog::FintechCaseSeed;

#[derive(Debug)]
pub(super) struct SeededEntityState {
    pub(super) book_names: Vec<String>,
    pub(super) desk_map: BTreeMap<String, EntityId>,
    pub(super) book_map: BTreeMap<String, EntityId>,
    pub(super) desks: Vec<EntityId>,
    pub(super) books: Vec<EntityId>,
    pub(super) accounts: Vec<EntityId>,
    pub(super) counterparties: Vec<EntityId>,
    pub(super) trades: Vec<EntityId>,
    pub(super) settlements: Vec<EntityId>,
    pub(super) cash_events: Vec<EntityId>,
    pub(super) audit_records: Vec<EntityId>,
    pub(super) instruments: Vec<EntityId>,
    pub(super) market_points: Vec<EntityId>,
    pub(super) risk_views: Vec<EntityId>,
    pub(super) limits: Vec<EntityId>,
    pub(super) breaches: Vec<EntityId>,
}

pub(super) fn seed_entities(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case_seeds: &[FintechCaseSeed],
) -> SeededEntityState {
    let desk_names = unique_names(case_seeds.iter().map(|seed| seed.desk_name));
    let book_names = unique_names(case_seeds.iter().map(|seed| seed.book_name));

    let desks = seed_desks(execution, runtime, &desk_names);
    let books = seed_books(execution, runtime, case_seeds, &book_names);
    let accounts = seed_accounts(execution, runtime, case_seeds);
    let counterparties = seed_counterparties(execution, runtime, case_seeds);
    let trades = seed_trades(execution, runtime, case_seeds);
    let settlements = seed_settlements(execution, runtime, case_seeds);
    let cash_events = seed_cash_events(execution, runtime, case_seeds);
    let audit_records = seed_audit_records(execution, runtime, case_seeds);
    let instruments = seed_instruments(execution, runtime, case_seeds);
    let market_points = seed_market_points(execution, runtime, case_seeds);
    let risk_views = seed_risk_views(execution, runtime, case_seeds);
    let limits = seed_limits(execution, runtime, case_seeds);
    let breaches = seed_breaches(execution, runtime, case_seeds);
    let desk_map = name_id_map(&desk_names, &desks);
    let book_map = name_id_map(&book_names, &books);

    SeededEntityState {
        book_names,
        desk_map,
        book_map,
        desks,
        books,
        accounts,
        counterparties,
        trades,
        settlements,
        cash_events,
        audit_records,
        instruments,
        market_points,
        risk_views,
        limits,
        breaches,
    }
}

fn unique_names<'a>(values: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut names = Vec::new();
    for value in values {
        if !names.iter().any(|existing| existing == value) {
            names.push(value.to_string());
        }
    }
    names
}

fn name_id_map(names: &[String], ids: &[EntityId]) -> BTreeMap<String, EntityId> {
    names
        .iter()
        .cloned()
        .zip(ids.iter().copied())
        .collect::<BTreeMap<_, _>>()
}
