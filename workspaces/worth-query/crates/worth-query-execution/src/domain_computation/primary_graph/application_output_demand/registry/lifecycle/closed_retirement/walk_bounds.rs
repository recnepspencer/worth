//! Actual mutation walk sites stay local when another occurrence grows.
use super::super::super::tests::{self as registry_tests, key, occurrence, record, OracleWalk};
use super::super::super::{DemandState, WorthQueryOutputCheckpoint, WorthQueryOutputProgress};
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use core::convert::Infallible;

impl DemandRegistryState {
    pub(in super::super::super) fn assert_walk_site<C>(site: usize, mut payer: impl FnMut() -> C)
    where
        C: OccurrenceSearchCharge<Refusal = Infallible>,
    {
        let ready = registry_tests::closed_retirement::ready();
        let crate::domain_computation::primary_graph::application_output_demand::registry::DemandState::Output(output) = ready else { panic!("Ready fixture") };
        let Some(crate::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint else { panic!("Ready fixture") };
        let receipts = registry_tests::walk_bounds::receipt_batch();
        let mut tally = 0;
        let mut contacts = Vec::new();
        for (neighbors, entries) in [(1, 1), (64, 4)] {
            let old = key("own", 1, 1);
            let newest = key("own", 2, 1);
            let dependent = key("dependent", 1, 2);
            let mut state = DemandRegistryState::default();
            let mut prior = record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "",
                )),
                0,
            );
            prior.framework_required_count = 1;
            state.records.insert(old.clone(), prior);
            let ready = || {
                DemandState::Output(WorthQueryOutputProgress::new(
                    WorthQueryOutputCheckpoint::Ready(completion.clone()),
                ))
            };
            state
                .records
                .insert(newest.clone(), record(occurrence(), ready(), 0));
            let mut row = record(occurrence(), ready(), 0);
            row.prerequisites.push(Arc::new(old.clone()));
            state.required_reserved_bytes = row.prerequisites.capacity()
                * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
            state.records.insert(dependent.clone(), row);
            for selected in [&old, &newest] {
                let member = state.prepare_required_member(selected).unwrap();
                state.install_required_member(member);
            }
            for generation in 1..=neighbors {
                state.records.insert(
                    key("larger-other", generation, 3),
                    record(occurrence(), DemandState::Admitted, 0),
                );
            }
            if site == 6 {
                let mut stopped = ready();
                if let DemandState::Output(output) = &mut stopped {
                    output.stop(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "",
                    ));
                }
                state.records.get_mut(&old).unwrap().state = stopped;
                state.records.get_mut(&newest).unwrap().state =
                    DemandState::Failed(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "",
                    ));
            }
            registry_tests::walk_bounds::add_custody(&mut state, &receipts[..entries], neighbors);
            let before = super::super::super::record_map::occurrence_rows_visited();
            let mut charge = payer();
            let done = match site {
                0 => {
                    // The dependent has moved its claim before retirement.
                    state
                        .records
                        .get_mut(&old)
                        .unwrap()
                        .framework_required_count = 0;
                    state
                        .records
                        .get_mut(&newest)
                        .unwrap()
                        .framework_required_count = 1;
                    state.records.get_mut(&dependent).unwrap().prerequisites[0] =
                        Arc::new(newest.clone());
                    state
                        .retire_row(&old, &mut Vec::new(), &mut charge)
                        .map(|_| ())
                }
                1 => state
                    .follow_newest(&dependent, &old, &mut charge)
                    .map(|_| ()),
                2 => newest_ready(&state.records, &old, &mut charge).map(|_| ()),
                3 => held(
                    &state.records,
                    &newest,
                    &state.records[&newest],
                    &mut charge,
                )
                .map(|_| ()),
                4 => state
                    .release_record_prerequisites(&dependent, &mut charge)
                    .map(|_| ()),
                5 => state.remove_required_member_if_released(&newest, &mut charge),
                6 => state
                    .revive_replaced_ready(&newest, &mut charge)
                    .map(|_| ()),
                _ => panic!("unknown walk site"),
            };
            match done {
                Ok(()) => {}
                Err(never) => match never {},
            }
            let visited = super::super::super::record_map::occurrence_rows_visited() - before;
            assert!(visited > 0, "site {site} was not reached");
            tally += 1;
            contacts.push(visited);
        }
        assert_eq!(contacts[0], contacts[1], "walk site {site}");
        assert_eq!(tally, 2);
    }
}

#[test]
fn registry_row_retirement_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(0, OracleWalk::after_effect);
}

#[test]
fn registry_follow_newest_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(1, OracleWalk::after_effect);
}

#[test]
fn registry_newest_ready_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(2, OracleWalk::after_effect);
}

#[test]
fn registry_held_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(3, OracleWalk::after_effect);
}

#[test]
fn registry_prerequisite_release_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(4, OracleWalk::after_effect);
}

#[test]
fn registry_member_release_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(5, OracleWalk::after_effect);
}

#[test]
fn registry_revive_ready_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(6, OracleWalk::after_effect);
}
