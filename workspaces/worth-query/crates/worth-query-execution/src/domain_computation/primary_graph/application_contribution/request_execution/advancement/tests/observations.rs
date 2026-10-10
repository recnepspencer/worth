//! Aggregate cost is read from the same advancement observer on every outcome.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::computation::{
    prepared_request::PreparedCompute,
    wave_dispatch::{dispatch, PreparedMember},
};

const COSTS: [u64; 4] = [3, 1, 4, 2];
const PREPARE: u64 = 2;
const COMPLETE: u64 = 1;

fn members(costs: &[u64], failure: Option<usize>) -> Vec<PreparedMember> {
    costs
        .iter()
        .enumerate()
        .map(|(rank, &units)| PreparedMember {
            requests: vec![PreparedCompute::declared_test_request(
                rank,
                units,
                failure == Some(rank),
            )],
        })
        .collect()
}

#[test]
fn aggregate_reports_add_stages_and_waves_and_keep_failed_advancements() {
    for posture in [
        Placement::Serial,
        Placement::Leased(NonZeroUsize::MIN),
        Placement::Leased(NonZeroUsize::new(2).unwrap()),
        Placement::Leased(NonZeroUsize::new(4).unwrap()),
    ] {
        let _restore = Restore(place(posture), bound(None));
        for failure in [None, Some(0), Some(1), Some(2), Some(3)] {
            observation::advancement_observations_on_this_thread_for_test();
            let root = request();
            let outcome = root
                .run_advancement(None, None, |_| {
                    charged_request(root.execution_request().unwrap(), PREPARE);
                    let slots = dispatch(&root, members(&COSTS, failure));
                    assert_eq!(slots.len(), COSTS.len());
                    drop(slots);
                    charged_request(root.execution_request().unwrap(), COMPLETE);
                    // An owner completion's semantic failure cannot erase compute already settled.
                    failure.map_or(Ok(()), Err)
                })
                .unwrap();
            assert_eq!(outcome, failure.map_or(Ok(()), Err));
            let observations = observation::advancement_observations_on_this_thread_for_test();
            let [observed] = observations.as_slice() else {
                panic!("one report: {observations:?}")
            };
            let dispatched = &COSTS;
            assert_eq!(
                observed.report.charged_work(),
                PREPARE + dispatched.iter().sum::<u64>() + COMPLETE
            );
            assert_eq!(
                observed.report.charged_span(),
                PREPARE + dispatched.iter().max().unwrap() + COMPLETE
            );
            assert!(observed.refusal.is_none());
            assert!(
                observed.applied.is_empty(),
                "compute alone performs no application effect"
            );
        }
        let root = request();
        observation::advancement_observations_on_this_thread_for_test();
        root.run_advancement(None, None, |_| {
            charged_request(root.execution_request().unwrap(), PREPARE);
            drop(dispatch(&root, members(&COSTS[..2], None)));
            drop(dispatch(&root, members(&COSTS[2..], None)));
            charged_request(root.execution_request().unwrap(), COMPLETE);
        })
        .unwrap();
        let observations = observation::advancement_observations_on_this_thread_for_test();
        assert_eq!(observations.len(), 1);
        assert_eq!(
            observations[0].report.charged_work(),
            PREPARE + COSTS.iter().sum::<u64>() + COMPLETE
        );
        assert_eq!(
            observations[0].report.charged_span(),
            PREPARE
                + COSTS[..2].iter().max().unwrap()
                + COSTS[2..].iter().max().unwrap()
                + COMPLETE
        );
    }
}

#[test]
fn admission_refusal_has_a_zero_report_and_a_panic_closes_observer_custody() {
    observation::advancement_observations_on_this_thread_for_test();
    let mut root = request();
    root.declared_work = 0;
    assert!(root
        .run_advancement(None, None, |_| panic!("refused before body"))
        .is_err());
    let refused = observation::advancement_observations_on_this_thread_for_test();
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].report.charged_work(), 0);
    assert_eq!(refused[0].report.charged_span(), 0);
    assert_eq!(
        refused[0].refusal,
        Some(WorthQueryAdvancementDenial::Resource(
            Resource::WorkExhausted
        ))
    );
    assert!(refused[0].applied.is_empty());
    let root = request();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        root.run_advancement(None, None, |_| {
            charged_request(root.execution_request().unwrap(), PREPARE);
            panic!("owner body panic")
        })
    }))
    .is_err());
    let panicked = observation::advancement_observations_on_this_thread_for_test();
    assert_eq!(panicked.len(), 1);
    assert_eq!(
        panicked[0].refusal,
        Some(WorthQueryAdvancementDenial::Panicked)
    );
    assert_eq!(panicked[0].report.charged_work(), PREPARE);
    root.run_advancement(None, None, |_| ()).unwrap();
    assert_eq!(
        observation::advancement_observations_on_this_thread_for_test().len(),
        1
    );
}

#[test]
fn member_host_refusals_do_not_add_thread_dependent_observations() {
    let mut serial = None;
    for posture in [
        Placement::Serial,
        Placement::Leased(NonZeroUsize::new(2).unwrap()),
    ] {
        let _restore = Restore(place(posture), bound(None));
        observation::advancement_observations_on_this_thread_for_test();
        let root = request();
        root.run_advancement(None, None, |_| {
            let members = (0..2)
                .map(|_| PreparedMember {
                    requests: vec![PreparedCompute::new(
                        0,
                        0,
                        |_| {
                            with_bootstrap_advancement(
                                super::super::super::test_placement::test_policy(
                                    NonZeroUsize::MIN,
                                    1 << 20,
                                ),
                                |_| (),
                            )
                        },
                        |_| 0,
                    )],
                })
                .collect();
            let computed = dispatch(&root, members);
            assert_eq!(computed.len(), 2);
            for member in computed {
                let refusal = member
                    .unwrap()
                    .into_slots()
                    .pop()
                    .unwrap()
                    .into_value::<Result<(), WorthQueryAdvancementDenial>>();
                assert_eq!(refusal, Err(WorthQueryAdvancementDenial::NestedOpening));
            }
        })
        .unwrap();
        let records = observation::advancement_observations_on_this_thread_for_test();
        let observed = records
            .into_iter()
            .map(|record| {
                (
                    record.refusal,
                    record.report.charged_work(),
                    record.report.charged_span(),
                    record.applied,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(observed.len(), 1, "only the owner reports the advancement");
        assert_eq!(observed, *serial.get_or_insert_with(|| observed.clone()));
    }
}
