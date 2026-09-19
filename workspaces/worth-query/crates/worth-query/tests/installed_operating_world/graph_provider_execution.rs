use std::sync::{Arc, Mutex};

use worth_proof::TransitionOutcome;
use worth_query::facade::domain;

use super::graph_read_material::graph_read_material;
use super::installed_operation_fixture::{
    configured_runtime_for_package, configured_runtime_for_partial_effect_package,
    configured_runtime_for_understated_cost_package, federated_operation_contract_drift_package,
    federated_package, federated_touch_package, FederatedOperationContractDrift, FederatedRead,
    GeometryDomain, ReadFamily,
};

mod call_affinity;
mod contract_admission;
mod provider_fixture;
mod resource_admission;
use provider_fixture::*;
#[test]
fn projection_receipt_without_query_material_denies_before_executor_contact() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut workspace = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(read_definition::<RemoteA>(
            "remote-a",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(RemoteA, ReceiptOnlyProvider)
        .graph_participation(read_definition::<RemoteB>(
            "remote-b",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(RemoteB, SelectiveProvider::new(&log, None))
        .workspace("graph-provider-receipt-only")
        .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, FederatedRead)
        .unwrap();
    let denial = match bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("receipt-only projection did not produce an exact denial"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryBoundExecutionDenialKind::GraphProvider
    );
    assert_eq!(denial.counters().graph_provider_contacts, 1);
    assert_eq!(denial.counters().executor_contacts, 0);
}
#[test]
fn every_graph_entrypoint_fails_at_its_exact_boundary_and_commit_precedes_touch() {
    for (name, failure, expected_log) in [
        ("project", FailAt::Project, vec!["project"]),
        ("observe", FailAt::Observe, vec!["project", "observe"]),
        (
            "commit",
            FailAt::Commit,
            vec!["project", "observe", "commit"],
        ),
        (
            "touch",
            FailAt::Touch,
            vec!["project", "observe", "commit", "touch"],
        ),
    ] {
        let log = Arc::new(Mutex::new(Vec::new()));
        let (a_failure, b_failure, commit_failure) = match failure {
            FailAt::Project | FailAt::Touch => (Some(failure), None, None),
            FailAt::Observe => (None, Some(failure), None),
            FailAt::Commit => (None, None, Some(failure)),
        };
        let mut workspace =
            configured_runtime_for_partial_effect_package(federated_touch_package::<
                RemoteA,
                RemoteB,
            >(true))
            .graph_participation(atomic_definition::<RemoteA>("remote-a"))
            .atomic_graph_participation_provider(
                RemoteA,
                SelectiveProvider::partial_effect(&log, a_failure),
                SharedCommit,
            )
            .graph_participation(atomic_definition::<RemoteB>("remote-b"))
            .atomic_graph_participation_provider(
                RemoteB,
                SelectiveProvider::partial_effect(&log, b_failure),
                SharedCommit,
            )
            .graph_commit_provider(
                SharedCommit,
                SelectiveProvider::partial_effect_commit(
                    &log,
                    commit_failure,
                    vec!["remote-a", "remote-b"],
                ),
            )
            .workspace(format!("graph-sabotage-{name}"))
            .unwrap();
        let installed = workspace.domain(GeometryDomain).unwrap();
        let bound = workspace
            .prepare_mutation_operating_world(workspace.current_world())
            .unwrap()
            .family(ReadFamily)
            .bind(&installed, FederatedRead)
            .unwrap();
        let denial = match bound
            .admit_execution_resources(
                (),
                crate::suite::installed_operation_fixture::partial_effect_execution_resource_request(
                ),
                &workspace,
            )
            .unwrap()
            .execute(&mut workspace)
        {
            TransitionOutcome::Denied(denial) => denial,
            _ => panic!("{name} sabotage did not produce an exact denial"),
        };
        assert_eq!(
            denial.kind(),
            &domain::WorthQueryBoundExecutionDenialKind::GraphProvider
        );
        assert_eq!(denial.counters().executor_contacts, 0);
        assert_eq!(
            denial.graph_receipts().len(),
            expected_log.len().saturating_sub(1),
            "{name} failure lost completed graph-provider evidence"
        );
        assert_eq!(*log.lock().unwrap(), expected_log);
    }
}

#[test]
fn read_only_participation_does_not_widen_the_mutating_commit_set() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut workspace = configured_runtime_for_partial_effect_package(federated_touch_package::<
        RemoteA,
        RemoteB,
    >(false))
    .graph_participation(atomic_definition::<RemoteA>("remote-a"))
    .atomic_graph_participation_provider(
        RemoteA,
        SelectiveProvider::partial_effect(&log, None),
        SharedCommit,
    )
    .graph_participation(read_definition::<RemoteB>(
        "remote-b",
        domain::WorthQueryGraphProjectionPosture::NativeProjection,
    ))
    .graph_participation_provider(RemoteB, SelectiveProvider::partial_effect(&log, None))
    .graph_commit_provider(
        SharedCommit,
        SelectiveProvider::partial_effect_commit(&log, None, vec!["remote-a"]),
    )
    .workspace("graph-selective-commit-set")
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, FederatedRead)
        .unwrap();
    assert_eq!(
        bound.commit_posture(),
        domain::WorthQueryBoundCommitPosture::Atomic
    );
    bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::partial_effect_execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    assert_eq!(
        *log.lock().unwrap(),
        ["project", "observe", "commit", "touch"]
    );
}

#[test]
fn graph_contract_insufficiency_denies_before_provider_contact() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let workspace = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(read_definition::<RemoteA>(
            "remote-a",
            domain::WorthQueryGraphProjectionPosture::NotRequired,
        ))
        .graph_participation_provider(RemoteA, SelectiveProvider::new(&log, None))
        .graph_participation(read_definition::<RemoteB>(
            "remote-b",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(RemoteB, SelectiveProvider::new(&log, None))
        .workspace("graph-contract-insufficient")
        .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let denial = match workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, FederatedRead)
    {
        Ok(_) => panic!("insufficient projection authority unexpectedly bound"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::GraphAuthorityInsufficient
    );
    assert!(log.lock().unwrap().is_empty());
    assert_eq!(denial.counters().graph_participation_lookups, 1);
    assert_eq!(denial.counters().graph_provider_contacts, 0);
}

fn atomic_definition<G>(role: &str) -> domain::WorthQueryGraphParticipationDefinition<G> {
    definition(
        role,
        domain::WorthQueryGraphProjectionPosture::NativeProjection,
        domain::WorthQueryGraphMutationPosture::TouchAndEffect,
        domain::WorthQueryGraphCommitPosture::AtomicAuthorityRequired,
    )
}

fn read_definition<G>(
    role: &str,
    projection: domain::WorthQueryGraphProjectionPosture,
) -> domain::WorthQueryGraphParticipationDefinition<G> {
    definition(
        role,
        projection,
        domain::WorthQueryGraphMutationPosture::NotRequired,
        domain::WorthQueryGraphCommitPosture::ReadOnly,
    )
}

fn definition<G>(
    role: &str,
    projection: domain::WorthQueryGraphProjectionPosture,
    mutation: domain::WorthQueryGraphMutationPosture,
    commit: domain::WorthQueryGraphCommitPosture,
) -> domain::WorthQueryGraphParticipationDefinition<G> {
    domain::WorthQueryGraphParticipationDefinition::new(
        role,
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection,
            mutation,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::ExternalBoundary,
            budget: domain::WorthQueryGraphBudgetPosture::ExternalBoundary,
            commit,
            failure: domain::WorthQueryGraphFailureTopology::BoundaryFailure,
        },
    )
}
