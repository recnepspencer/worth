//! Actual mutation walk sites stay local when another occurrence grows.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::{
    key, occurrence, record,
};
use core::convert::Infallible;

impl DemandRegistryState {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn assert_completed_walk_sites<
        C,
    >(
        mut payer: impl FnMut() -> C,
    ) where
        C: OccurrenceSearchCharge<Refusal = Infallible>,
    {
        let ready = crate::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
        let crate::domain_computation::primary_graph::application_output_demand::registry::DemandState::Output(output) = ready else { panic!("Ready fixture") };
        let Some(crate::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint else { panic!("Ready fixture") };
        let mut tallies = [0; 7];
        for (site, tally) in tallies.iter_mut().enumerate() {
            let mut contacts = Vec::new();
            for neighbors in [1, 64] {
                let old = key("own", 1, 1);
                let newest = key("own", 2, 1);
                let dependent = key("dependent", 1, 2);
                let mut state = DemandRegistryState::default();
                let mut prior = record(occurrence(), super::super::super::DemandState::Failed(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Superseded, "")), 0);
                prior.framework_required_count = 1;
                state.records.insert(old.clone(), prior);
                let ready = || {
                    super::super::super::DemandState::Output(
                        super::super::super::WorthQueryOutputProgress::new(
                            super::super::super::WorthQueryOutputCheckpoint::Ready(
                                completion.clone(),
                            ),
                        ),
                    )
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
                        record(occurrence(), super::super::super::DemandState::Admitted, 0),
                    );
                }
                if site == 6 {
                    let mut stopped = ready();
                    if let super::super::super::DemandState::Output(output) = &mut stopped {
                        output.stop(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Superseded, ""));
                    }
                    state.records.get_mut(&old).unwrap().state = stopped;
                    state.records.get_mut(&newest).unwrap().state = super::super::super::DemandState::Failed(
                        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Superseded, ""));
                }
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
                *tally += 1;
                contacts.push(visited);
            }
            assert_eq!(contacts[0], contacts[1], "walk site {site}");
        }
        assert!(tallies.into_iter().all(|visits| visits > 0));
    }
}
