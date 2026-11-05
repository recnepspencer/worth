//! Cancellation is signalled inside the real refresh pass, at every check.
use crate::domain_computation::primary_graph::tests::index_maintenance_interruptions::*;
use worth_execution::RequestConsultObservationForTest;
#[test]
fn refresh_preserves_the_stop_at_every_counted_consult() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_world(&[("counted-refresh", WorthQueryPrincipalMappingStatus::Enabled)]);
    let handle = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .integration_handle_for_test();
    let setup_host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let setup = ExecutionRequest::serial(&setup_host);
    let discard = || {
        (*handle.source_owner_for_test()).with_runtime(&setup, |runtime| {
            for index in (*handle.primary_index_ids_for_test()).iter() {
                runtime
                    .index_authority()
                    .discard_generations(
                        worth_relational::facade::indexes::DerivedIndexDiscardRequest::all_bases(
                            *index,
                        ),
                    )
                    .unwrap();
            }
        })
    };
    let run = |k: Option<usize>| {
        discard();
        let source = CancellationSource::new();
        let cancel = source.clone();
        let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), source.token(), None);
        let execution = ExecutionRequest::serial(&host);
        let mut observation = None;
        let result = handle.execute_mutation_with_index_refresh(&execution, |runtime| {
            commit_empty(&setup, runtime, "main");
            observation = Some(
                RequestConsultObservationForTest::install_on_this_thread_for_test(move |at| {
                    if Some(at) == k {
                        cancel.cancel();
                    }
                }),
            );
            Ok::<_, ()>(())
        });
        let count = observation.as_ref().unwrap().count_for_test();
        drop(observation);
        (result, count)
    };
    let (healthy, count) = run(None);
    healthy.unwrap().unwrap();
    assert!(count > 2);
    eprintln!("refresh consults={count}");
    for k in 1..=count + 1 {
        let (result, _) = run(Some(k));
        if k > count {
            result.unwrap().unwrap();
        } else {
            assert_eq!(
                refresh_stop(&result.unwrap_err()),
                Some(Err(Control::Cancelled)),
                "consult {k}"
            );
        }
    }
}
