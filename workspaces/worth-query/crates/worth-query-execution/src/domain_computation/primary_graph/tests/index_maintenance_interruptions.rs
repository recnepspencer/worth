use super::fixture::installed_world;
use super::index_refresh::commit_empty;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;

fn index_currency_stop(cancelled: bool) {
    let world = installed_world(&[("currency-stop", WorthQueryPrincipalMappingStatus::Enabled)]);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let setup_host = crate::test_request_policy::serial_request();
    let setup = ExecutionRequest::serial(&setup_host);
    handle.source_owner.with_runtime_mut(&setup, |runtime| {
        commit_empty(&setup, runtime, "main");
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let source = CancellationSource::new();
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(64 * 1024 * 1024),
            source.token(),
            None,
        );
        let execution = ExecutionRequest::serial(&host);
        if cancelled {
            source.cancel();
            let denied = handle
                .ensure_primary_indexes_for_basis(&execution, runtime, &basis)
                .unwrap_err();
            assert_eq!(currency_stop(denied), Some(Err(Control::Cancelled)));
            return;
        }
        // Both twins reconstruct the same cold inventory. The installed
        // world may already carry generations before this first measurement.
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
        let (enough, report) = setup
            .run(ExecutionWorkCeiling::new(Some(100_000)), |_| {
                handle.ensure_primary_indexes_for_basis(&setup, runtime, &basis)
            })
            .unwrap();
        enough.unwrap();
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
        assert!(report.charged_work() > 0);
        let (short, _) = setup
            .run(
                ExecutionWorkCeiling::new(Some(report.charged_work() - 1)),
                |_| handle.ensure_primary_indexes_for_basis(&setup, runtime, &basis),
            )
            .unwrap();
        assert_eq!(
            currency_stop(short.unwrap_err()),
            Some(Ok(Kind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: None,
            }))
        );
    });
}

fn mutation_index_refresh_stop(cancelled: bool) {
    let world = installed_world(&[("refresh-stop", WorthQueryPrincipalMappingStatus::Enabled)]);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let source = CancellationSource::new();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        source.token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    if cancelled {
        let denied = handle
            .execute_mutation_with_index_refresh(&execution, |runtime| {
                commit_empty(&execution, runtime, "main");
                source.cancel();
                Ok::<_, ()>(())
            })
            .unwrap_err();
        assert_eq!(refresh_stop(&denied), Some(Err(Control::Cancelled)));
        return;
    }
    let setup_host = crate::test_request_policy::serial_request();
    let setup = ExecutionRequest::serial(&setup_host);
    let (enough, report) = setup
        .run(ExecutionWorkCeiling::new(Some(100_000)), |_| {
            handle.execute_mutation_with_index_refresh(&setup, |runtime| {
                commit_empty(&setup, runtime, "main");
                Ok::<_, ()>(())
            })
        })
        .unwrap();
    enough.unwrap().unwrap();
    assert!(report.charged_work() > 0);
    let (short, _) = setup
        .run(
            ExecutionWorkCeiling::new(Some(report.charged_work() - 1)),
            |_| {
                handle.execute_mutation_with_index_refresh(&setup, |runtime| {
                    commit_empty(&setup, runtime, "main");
                    Ok::<_, ()>(())
                })
            },
        )
        .unwrap();
    assert_eq!(
        refresh_stop(&short.unwrap_err()),
        Some(Ok(Kind::ExecutionResource {
            denial: Resource::WorkExhausted,
            partition_identity: None,
            policy_ancestor: None,
        }))
    );
}

fn currency_stop(
    denial: super::super::index_currency::WorthQueryPrimaryIndexCurrencyDenial,
) -> Option<Result<Kind, Control>> {
    use super::super::index_currency::WorthQueryPrimaryIndexCurrencyDenial as Denial;
    match denial {
        Denial::Execution(stop) => Some(crate::facade::primary_graph::relational_execution_kind(
            stop, None,
        )),
        Denial::Basis(_) | Denial::IndexUnavailable(_) => None,
    }
}
fn refresh_stop(
    denial: &super::super::WorthQueryPrimaryGraphIndexRefreshDenial,
) -> Option<Result<Kind, Control>> {
    use super::super::WorthQueryPrimaryGraphIndexRefreshDenialKind as Denial;
    match denial.kind() {
        Denial::Execution(stop) => Some(stop),
        Denial::MissingCommittedMutation | Denial::IndexBuildRejected => None,
    }
}

#[test]
fn currency_cancelled() {
    index_currency_stop(true);
}
#[test]
fn currency_one_unit_short() {
    index_currency_stop(false);
}
#[test]
fn refresh_cancelled() {
    mutation_index_refresh_stop(true);
}
#[test]
fn refresh_one_unit_short() {
    mutation_index_refresh_stop(false);
}

mod consumers;

mod refresh_consults;
