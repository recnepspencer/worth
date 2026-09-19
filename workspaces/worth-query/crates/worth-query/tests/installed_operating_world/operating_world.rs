use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use worth_query::facade::domain;

use super::installed_operation_fixture::{
    configured_runtime_for_package, configured_runtime_for_partial_effect_package,
    federated_package, federated_touch_package, required_domain_runtime, workspace, FederatedRead,
    GeometryDomain, ReadFamily, ReadVertex,
};

mod provider_fixture;
use provider_fixture::*;

#[test]
fn required_domain_is_resolved_and_retained_by_the_bound_capability() {
    let workspace = required_domain_runtime(true)
        .workspace("operating-world-required-domain")
        .unwrap();
    let rebuild = workspace.verify_domain_execution_index_rebuild();
    assert!(rebuild.is_equivalent());
    assert_eq!(rebuild.operation_required_domain_count(), 1);
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();

    assert_eq!(
        bound.required_domain_roles().collect::<Vec<_>>(),
        ["auxiliary"]
    );
}

#[test]
fn missing_required_domain_denies_before_graph_or_execution_work() {
    let workspace = required_domain_runtime(false)
        .workspace("operating-world-missing-required-domain")
        .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let denial = match workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
    {
        Ok(_) => panic!("missing exact domain authority must deny binding"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::RequiredDomainNotInstalled
    );
    assert_eq!(denial.counters().required_domain_lookups, 1);
    assert_eq!(denial.counters().graph_provider_contacts, 0);
    assert_eq!(denial.counters().planning_steps, 0);
}

#[test]
fn one_root_mints_equivalent_non_detachable_bound_authority() {
    let workspace = workspace("operating-world", false).unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let first = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&domain, ReadVertex)
        .unwrap();
    let second = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&domain, ReadVertex)
        .unwrap();
    assert_eq!(first.binding_identity(), second.binding_identity());
    assert_eq!(
        first.commit_posture(),
        domain::WorthQueryBoundCommitPosture::ReadOnly
    );
    assert_eq!(first.graph_roles().count(), 0);
}

#[test]
fn foreign_domain_denies_before_graph_binding_or_provider_contact() {
    let owner = workspace("operating-world-owner", false).unwrap();
    let foreign = workspace("operating-world-foreign", false).unwrap();
    let foreign_domain = foreign.domain(GeometryDomain).unwrap();
    let denial = match owner
        .observe_operating_world(owner.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&foreign_domain, ReadVertex)
    {
        Ok(_) => panic!("foreign domain authority must not bind"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::DomainAuthority
    );
    assert_eq!(denial.counters().graph_binding_lookups, 0);
    assert_eq!(denial.counters().graph_provider_contacts, 0);
    assert_eq!(denial.counters().planning_steps, 0);
}

#[test]
fn read_only_operation_does_not_claim_or_contact_adapter_commit_authority() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(atomic_definition::<RemoteA>("remote-a"))
        .atomic_graph_participation_provider(
            RemoteA,
            Provider::effect_free(Arc::clone(&contacts)),
            SharedCommit,
        )
        .graph_participation(atomic_definition::<RemoteB>("remote-b"))
        .atomic_graph_participation_provider(
            RemoteB,
            Provider::effect_free(Arc::clone(&contacts)),
            SharedCommit,
        )
        .graph_commit_provider(SharedCommit, Provider::effect_free(Arc::clone(&contacts)))
        .workspace("operating-world-shared-commit")
        .unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    assert_eq!(
        workspace
            .verify_domain_execution_index_rebuild()
            .operation_graph_participation_count(),
        2
    );
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&domain, FederatedRead)
        .unwrap();
    assert_eq!(
        bound.commit_posture(),
        domain::WorthQueryBoundCommitPosture::ReadOnly
    );
    assert_eq!(
        bound.graph_roles().collect::<Vec<_>>(),
        ["remote-a", "remote-b"]
    );
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
    let executed = bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    assert_eq!(executed.graph_receipts().len(), 2);
    assert!(executed
        .graph_receipts()
        .iter()
        .find(|receipt| receipt.role() == "remote-a")
        .is_some_and(|receipt| receipt.has_projection_material()));
    assert_eq!(
        executed.warnings(),
        [domain::WorthQueryOperationExecutionWarning::Advisory(
            "remote-a-projected-rows=1".into()
        )]
    );
    assert_eq!(executed.counters().graph_provider_contacts, 2);
    assert_eq!(contacts.load(Ordering::Relaxed), 2);
}

#[test]
fn independent_equal_role_providers_deny_before_provider_contact() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let workspace = configured_runtime_for_partial_effect_package(federated_touch_package::<
        RemoteA,
        RemoteB,
    >(true))
    .graph_participation(atomic_definition::<RemoteA>("remote-a"))
    .atomic_graph_participation_provider(
        RemoteA,
        Provider::partial_effects(Arc::clone(&contacts)),
        SharedCommit,
    )
    .graph_participation(atomic_definition::<RemoteB>("remote-b"))
    .atomic_graph_participation_provider(
        RemoteB,
        Provider::partial_effects(Arc::clone(&contacts)),
        OtherCommit,
    )
    .graph_commit_provider(
        SharedCommit,
        Provider::partial_effects(Arc::clone(&contacts)),
    )
    .graph_commit_provider(
        OtherCommit,
        Provider::partial_effects(Arc::clone(&contacts)),
    )
    .workspace("operating-world-split-commit")
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let denial = match workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, FederatedRead)
    {
        Ok(_) => panic!("independent commit owners must not mint atomic authority"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::CompensationUndeclared
    );
    assert_eq!(denial.counters().graph_participation_lookups, 2);
    assert_eq!(denial.counters().graph_provider_contacts, 0);
    assert_eq!(denial.counters().planning_steps, 0);
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}

#[test]
fn separately_committed_graphs_deny_without_domain_aftermath_compensation() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let workspace = configured_runtime_for_partial_effect_package(federated_touch_package::<
        RemoteA,
        RemoteB,
    >(true))
    .graph_participation(atomic_definition::<RemoteA>("remote-a"))
    .atomic_graph_participation_provider(
        RemoteA,
        Provider::partial_effects(Arc::clone(&contacts)),
        SharedCommit,
    )
    .graph_participation(atomic_definition::<RemoteB>("remote-b"))
    .atomic_graph_participation_provider(
        RemoteB,
        Provider::partial_effects(Arc::clone(&contacts)),
        OtherCommit,
    )
    .graph_commit_provider(
        SharedCommit,
        Provider::partial_effects(Arc::clone(&contacts)),
    )
    .graph_commit_provider(
        OtherCommit,
        Provider::partial_effects(Arc::clone(&contacts)),
    )
    .workspace("operating-world-compensated-commit")
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    // Domain operations cannot yet carry aftermath from an installed declaration
    // (the retired caller-authored domain installer is closed). Separate-commit
    // mutation therefore fails closed until that honest path exists.
    let denial = match workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, FederatedRead)
    {
        Ok(_) => panic!("separate commits must deny without domain aftermath compensation"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::CompensationUndeclared
    );
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}

#[test]
fn same_role_lookalike_cannot_replace_the_exact_attached_graph_marker() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let result = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(atomic_definition::<RemoteALookalike>("remote-a"))
        .atomic_graph_participation_provider(
            RemoteALookalike,
            Provider::effect_free(Arc::clone(&contacts)),
            SharedCommit,
        )
        .graph_participation(atomic_definition::<RemoteB>("remote-b"))
        .atomic_graph_participation_provider(
            RemoteB,
            Provider::effect_free(Arc::clone(&contacts)),
            SharedCommit,
        )
        .graph_commit_provider(SharedCommit, Provider::effect_free(Arc::clone(&contacts)))
        .workspace("operating-world-graph-lookalike")
        .unwrap();
    let installed_domain = result.domain(GeometryDomain).unwrap();
    let denial = match result
        .observe_operating_world(result.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, FederatedRead)
    {
        Ok(_) => panic!("a same-role graph lookalike must not satisfy an exact attachment"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::GraphParticipationNotInstalled
    );
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}

fn atomic_definition<G>(role: &str) -> domain::WorthQueryGraphParticipationDefinition<G> {
    domain::WorthQueryGraphParticipationDefinition::new(
        role,
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection: domain::WorthQueryGraphProjectionPosture::NativeProjection,
            mutation: domain::WorthQueryGraphMutationPosture::TouchAndEffect,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::ExternalBoundary,
            budget: domain::WorthQueryGraphBudgetPosture::ExternalBoundary,
            commit: domain::WorthQueryGraphCommitPosture::AtomicAuthorityRequired,
            failure: domain::WorthQueryGraphFailureTopology::BoundaryFailure,
        },
    )
}
