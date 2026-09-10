use std::sync::{Arc, Mutex};

use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, installed};

use super::provider_fixture::{RemoteA, RemoteB, SelectiveProvider, SharedCommit};
use super::{atomic_definition, read_definition};
use crate::suite::installed_operation_fixture::{
    configured_runtime_for_package, configured_runtime_for_partial_effect_package,
    federated_package, federated_touch_package, FederatedRead, GeometryDomain, ReadFamily,
};

#[test]
fn commit_provider_mismatch_denies_before_any_graph_contact() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let workspace = configured_runtime_for_partial_effect_package(federated_touch_package::<
        RemoteA,
        RemoteB,
    >(true))
    .graph_participation(atomic_definition::<RemoteA>("remote-a"))
    .atomic_graph_participation_provider(
        RemoteA,
        SelectiveProvider::partial_effect(&log, None),
        SharedCommit,
    )
    .graph_participation(atomic_definition::<RemoteB>("remote-b"))
    .atomic_graph_participation_provider(
        RemoteB,
        SelectiveProvider::partial_effect(&log, None),
        SharedCommit,
    )
    .graph_commit_provider(
        SharedCommit,
        SelectiveProvider::commit_with_support(
            &log,
            vec!["remote-a", "remote-b"],
            mismatched_partial_effect_provider_resource_support(),
        ),
    )
    .workspace("graph-commit-resource-mismatch")
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, FederatedRead)
        .unwrap();

    let TransitionOutcome::Denied(denial) = bound.admit_execution_resources(
        (),
        crate::suite::installed_operation_fixture::partial_effect_execution_resource_request(),
        &workspace,
    ) else {
        panic!("commit-provider support mismatch must deny")
    };

    assert_resource_mismatch(&denial, "commit group");
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn graph_provider_mismatch_denies_before_any_graph_contact() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let workspace = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(read_definition::<RemoteA>(
            "remote-a",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(
            RemoteA,
            SelectiveProvider::new_with_support(&log, mismatched_provider_resource_support()),
        )
        .graph_participation(read_definition::<RemoteB>(
            "remote-b",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(RemoteB, SelectiveProvider::new(&log, None))
        .workspace("graph-provider-resource-mismatch")
        .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, FederatedRead)
        .unwrap();

    let TransitionOutcome::Denied(denial) = bound.admit_execution_resources(
        (),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &workspace,
    ) else {
        panic!("graph-provider support mismatch must deny")
    };

    assert_resource_mismatch(&denial, "graph role");
    assert!(log.lock().unwrap().is_empty());
}

fn assert_resource_mismatch(
    denial: &installed::operation::WorthQueryExecutionResourceAdmissionDenial,
    subject: &str,
) {
    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::
            CancellationSafePointUnsupported
    );
    assert!(denial.detail().contains(subject));
    assert_eq!(denial.counters().provider_session_mints, 0);
}

fn mismatched_provider_resource_support() -> domain::WorthQueryExecutionResourceSupport {
    mismatched_provider_resource_support_with(domain::WorthQueryPartialEffectPosture::EffectFree)
}

fn mismatched_partial_effect_provider_resource_support(
) -> domain::WorthQueryExecutionResourceSupport {
    mismatched_provider_resource_support_with(
        domain::WorthQueryPartialEffectPosture::PartialEffectsMayRemain,
    )
}

fn mismatched_provider_resource_support_with(
    partial_effect_posture: domain::WorthQueryPartialEffectPosture,
) -> domain::WorthQueryExecutionResourceSupport {
    domain::WorthQueryExecutionResourceSupport::new(
        domain::WorthQueryExecutionProviderFamily::new("fixture-provider").unwrap(),
        domain::WorthQueryExecutionAccessProductFamily::new("fixture-access").unwrap(),
        domain::WorthQueryExecutionAllocatorFamily::new("fixture-arena").unwrap(),
        domain::WorthQueryExecutionResourceEnvelope::bounded(
            1_000_000,
            1_000_000,
            domain::WorthQueryExecutionMode::Synchronous,
            domain::WorthQueryCancellationSafePointFamily::new("incompatible-safe-point").unwrap(),
        )
        .with_partial_effect_posture(partial_effect_posture),
        std::sync::Arc::new(
            domain::WorthQueryFixedExecutionCapacity::mint("mismatched-graph-provider", 8).unwrap(),
        ),
    )
}
