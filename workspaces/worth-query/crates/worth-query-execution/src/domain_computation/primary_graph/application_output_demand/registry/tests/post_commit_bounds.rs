//! Locally sealed no-refusal payers cannot widen an occurrence search.
use super::super::refreshed_rejoin::{
    awaited_by_stale_owner, newest_of_occurrence, occurrence_rows, OccurrenceSearchCharge,
};
use super::*;
use core::convert::Infallible;

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn check<C>(
    mut payer: impl FnMut() -> C,
) where
    C: OccurrenceSearchCharge<Refusal = Infallible>,
{
    for search in 0..3 {
        let mut contacts = Vec::new();
        for neighbors in [1, 64] {
            let mut state = DemandRegistryState::default();
            let selected = key("own", 1, 1);
            for generation in 1..=3 {
                state.records.insert(
                    key("own", generation, 1),
                    record(occurrence(), DemandState::Admitted, 0),
                );
            }
            for generation in 1..=neighbors {
                state.records.insert(
                    key("larger-other", generation, 2),
                    record(occurrence(), DemandState::Admitted, 0),
                );
            }
            let before = super::super::record_map::occurrence_rows_visited();
            let mut charge = payer();
            match search {
                0 => assert_eq!(
                    super::super::refreshed_rejoin::infallible(occurrence_rows(
                        &state.records,
                        &selected,
                        &mut charge
                    ))
                    .count(),
                    3
                ),
                1 => assert_eq!(
                    super::super::refreshed_rejoin::infallible(newest_of_occurrence(
                        &state.records,
                        &selected,
                        &mut charge
                    )),
                    Some(key("own", 3, 1))
                ),
                2 => assert!(!super::super::refreshed_rejoin::infallible(
                    awaited_by_stale_owner(&state.records, &selected, &mut charge)
                )),
                _ => panic!("unknown occurrence search"),
            }
            contacts.push(super::super::record_map::occurrence_rows_visited() - before);
        }
        assert_eq!(contacts, vec![3, 3], "search {search}");
    }
    DemandRegistryState::assert_completed_walk_sites(&mut payer);
    DemandRegistryState::assert_completed_custody_walk(payer);
}
