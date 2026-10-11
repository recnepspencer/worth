//! Exact snapshot custody still settles after the caller unwinds on a refusal.
use super::WorthQueryApplicationBasisRegistry;
use crate::domain_computation::primary_graph::{
    application_contribution::WorthQueryManagedComputationResourceDenial as Resource,
    index_currency::WorthQueryPrimaryIndexCurrencyDenial as Currency,
    tests::fixture::installed_authorization_world,
};
use crate::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, WorkCeilingDenial};
#[test]
fn refused_maintenance_drops_every_owner_without_panicking() {
    let filter = concat!(module_path!(), "::refused_maintenance_cleanup_probe")
        .split_once("::")
        .unwrap()
        .1;
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", filter, "--nocapture"])
        .env("WORTH_QUERY_MAINTENANCE_CLEANUP_PROBE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "cleanup child failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
}
#[test]
fn refused_maintenance_cleanup_probe() {
    if std::env::var_os("WORTH_QUERY_MAINTENANCE_CLEANUP_PROBE").is_none() {
        return;
    }
    let world = installed_authorization_world(true);
    let graph = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let basis = graph.with_runtime_mut(&execution, |runtime| {
        let (_, before) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let transaction = runtime
            .begin_branch_transaction(
                &before,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        let committed = transaction.commit(&execution, runtime).unwrap();
        crate::relational_snapshot_release::release_query_snapshot(runtime, &committed.snapshot);
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
    });
    let registry = WorthQueryApplicationBasisRegistry::default();
    let observer = registry.observer();
    let lease = registry
        .register(&execution, basis, graph.clone())
        .unwrap_or_else(|_| panic!("real exact basis must register"));
    assert_eq!(observer.observe().active(), 1);
    let stopped = execution.run(ExecutionWorkCeiling::new(0), |_| {
        graph.with_runtime_mut(&execution, |runtime| {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let refusal = graph
                .ensure_primary_indexes_for_basis(&execution, runtime, &basis)
                .unwrap_err();
            let Currency::Execution(cause) = refusal else {
                panic!("maintenance keeps the native request stop")
            };
            assert_eq!(
                cause,
                Ok(Kind::ExecutionResource {
                    denial: Resource::WorkExhausted,
                    partition_identity: None,
                    policy_ancestor: None
                })
            );
            panic!("caller unwinds after the real typed maintenance refusal");
        });
    });
    assert!(matches!(stopped, Err(WorkCeilingDenial::Panicked)));
    drop(lease);
    assert_eq!(observer.observe().active(), 0);
    drop(registry);
    drop(graph);
    drop(world);
}
