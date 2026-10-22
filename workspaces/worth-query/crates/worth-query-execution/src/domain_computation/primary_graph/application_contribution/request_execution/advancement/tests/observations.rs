//! The original advancement observer preserves refusal, panic and owner-only custody.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::computation::{
    prepared_request::PreparedCompute,
    wave_dispatch::{dispatch, PreparedMember},
};

const PREPARE: u64 = 2;

#[test]
fn admission_refusal_has_a_zero_report_and_a_panic_closes_observer_custody() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    observation::advancement_observations_on_this_thread_for_test();
    let mut root = request();
    root.declared_work = Some(0);
    assert!(root
        .run_advancement(None, None, |_| panic!("refused before body"))
        .is_err());
    let refused = observation::advancement_observations_on_this_thread_for_test();
    assert_eq!(refused.len(), 1);
    assert_eq!(
        refused[0].workers, None,
        "a refused admission ran no dispatch scope"
    );

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

    root.run_advancement(None, None, |_| ()).unwrap();
    assert_eq!(
        observation::advancement_observations_on_this_thread_for_test().len(),
        1
    );
}

#[test]
fn member_host_refusals_do_not_add_thread_dependent_observations() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
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
                    requests: vec![PreparedCompute::new(|_| {
                        with_bootstrap_advancement(
                            super::super::super::test_placement::test_policy(
                                NonZeroUsize::MIN,
                                None,
                            ),
                            |_| (),
                        )
                    })],
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
