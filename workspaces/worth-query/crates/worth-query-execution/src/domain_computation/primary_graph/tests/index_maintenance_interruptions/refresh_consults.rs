//! Cancellation is signalled inside the real refresh pass, at every check.
use super::*;
use worth_execution::RequestConsultObservation;
#[test]
fn refresh_preserves_the_stop_at_every_counted_consult() {
    let world = installed_world(&[("counted-refresh", WorthQueryPrincipalMappingStatus::Enabled)]);
    let handle = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    let setup_host = crate::test_request_policy::serial_request();
    let setup = ExecutionRequest::serial(&setup_host);
    let discard = || {
        handle.source_owner.with_runtime_mut(&setup, |runtime| {
            for index in handle.primary_index_ids.iter() {
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
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(64 * 1024 * 1024),
            source.token(),
            None,
        );
        let execution = ExecutionRequest::serial(&host);
        let mut observation = None;
        let result = handle.execute_mutation_with_index_refresh(&execution, |runtime| {
            commit_empty(&setup, runtime, "main");
            observation = Some(RequestConsultObservation::begin(move |at| {
                if Some(at) == k {
                    cancel.cancel();
                }
            }));
            Ok::<_, ()>(())
        });
        let count = observation.as_ref().unwrap().count();
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
