use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, foundation, runtime};
use worth_signal::facade::runtime::SignalConditionalEvaluationBudget;

use super::{conditional_node_contract, installed_operation_fixture};
use crate::support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn selected_product_turnover_releases_query_and_signal_retention_together() {
    let node = conditional_node_contract::resource_lifecycle_node();
    let installation = installed_operation_fixture::conditional_installation(&node);
    let harness = PublicBridgeRuntimeHarness::new();
    let resources = runtime::WorthQueryConditionalExecutionResources::new(
        runtime::WorthQueryConditionalEvaluationCacheBudget::bounded(2, 1024 * 1024).unwrap(),
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: 2,
            maximum_retained_bytes: 512 * 1024 * 1024,
            maximum_attempt_visits: 8_000_000,
        },
    );
    let mut workspace = installed_operation_fixture::conditional_resource_workspace(
        "conditional-resource-lifecycle",
        node,
        installation,
        &harness,
        resources,
    )
    .unwrap();
    let installed = workspace
        .domain(installed_operation_fixture::GeometryDomain)
        .unwrap();

    let first = create_sibling(&workspace, "conditional-resource-first", 1);
    execute_selected(&mut workspace, &installed, &first);
    execute_default(&mut workspace, &installed);
    let full = workspace
        .conditional_evaluation_resource_observation()
        .unwrap();
    assert_eq!(full.query_retained_entries(), 2);
    assert_eq!(full.signal_retained_slots(), 2);
    assert!(full.query_retained_bytes() > 0);
    assert!(full.signal_retained_bytes() > 0);

    let reused = execute_selected(&mut workspace, &installed, &first);
    let warm = workspace
        .conditional_evaluation_resource_observation()
        .unwrap();
    assert_eq!(warm.query_cache_hits(), full.query_cache_hits() + 1);
    assert_eq!(warm.query_cache_misses(), full.query_cache_misses());
    assert_eq!(warm.query_retained_bytes(), full.query_retained_bytes());
    assert_eq!(warm.signal_retained_bytes(), full.signal_retained_bytes());
    assert_eq!(reused.slot_reuse_hits, 1);
    assert_eq!(reused.source_admission_attempts, 0);
    assert_eq!(reused.compute_contacts, 0);

    let second = create_sibling(&workspace, "conditional-resource-second", 2);
    execute_selected(&mut workspace, &installed, &second);
    let turned = workspace
        .conditional_evaluation_resource_observation()
        .unwrap();
    assert_eq!(turned.query_retained_entries(), 2);
    assert_eq!(turned.signal_retained_slots(), 2);
    assert_eq!(turned.query_cache_misses(), warm.query_cache_misses() + 1);
    assert_eq!(
        turned.query_cache_evictions(),
        warm.query_cache_evictions() + 1
    );
    assert_eq!(turned.query_retained_bytes(), warm.query_retained_bytes());
    assert_eq!(turned.signal_retained_bytes(), warm.signal_retained_bytes());

    execute_default(&mut workspace, &installed);
    let retried = workspace
        .conditional_evaluation_resource_observation()
        .unwrap();
    assert_eq!(retried.query_retained_entries(), 2);
    assert_eq!(retried.signal_retained_slots(), 2);
    assert_eq!(
        retried.query_cache_misses(),
        turned.query_cache_misses() + 1
    );
    assert_eq!(
        retried.query_cache_evictions(),
        turned.query_cache_evictions() + 1
    );
    assert_eq!(
        retried.query_retained_bytes(),
        turned.query_retained_bytes()
    );
    assert_eq!(
        retried.signal_retained_bytes(),
        turned.signal_retained_bytes()
    );
}

fn create_sibling(
    workspace: &runtime::WorthQueryWorkspace,
    name: &str,
    ordinal: usize,
) -> runtime::ProductBranchIdentity {
    let source_world = workspace.observe_operating_world().unwrap();
    let source = source_world.product_branch().unwrap();
    let creation = runtime::ProductBranchCreationIntent::from_source(
        name,
        runtime::ProductBranchCreationPlans::new(
            runtime::RelationalBranchCreationPlan::ForkExact {
                target: runtime::BranchId(format!("conditional-resource-truth-{ordinal}")),
            },
            runtime::SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let cancellation = runtime::RuntimeWorldCancellationSource::new();
    let outcome = workspace
        .create_product_branch(source, creation, &cancellation.token())
        .unwrap();
    let runtime::RuntimeWorldBranchCreationOutcome::Performed(observation) = outcome else {
        panic!("World must publish the selected sibling: {outcome:?}")
    };
    observation.branch_identity().clone()
}

fn execute_default(
    workspace: &mut runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<
        installed_operation_fixture::GeometryDomain,
    >,
) -> ConditionalExecutionObservation {
    let bound = bind_default(workspace, installed);
    execute_bound(workspace, bound)
}

fn execute_selected(
    workspace: &mut runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<
        installed_operation_fixture::GeometryDomain,
    >,
    selected: &runtime::ProductBranchIdentity,
) -> ConditionalExecutionObservation {
    let bound = bind_selected(workspace, installed, selected);
    execute_bound(workspace, bound)
}

fn bind_default(
    workspace: &runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<
        installed_operation_fixture::GeometryDomain,
    >,
) -> domain::WorthQueryBoundDomainOperation<
    installed_operation_fixture::GeometryDomain,
    installed_operation_fixture::ConditionalResourceOperation,
    installed_operation_fixture::ConditionalResourceFamily,
    foundation::ObservationLaneWitness,
> {
    workspace
        .observe_operating_world()
        .unwrap()
        .family(installed_operation_fixture::ConditionalResourceFamily)
        .bind(
            installed,
            installed_operation_fixture::ConditionalResourceOperation,
        )
        .unwrap()
}

fn bind_selected(
    workspace: &runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<
        installed_operation_fixture::GeometryDomain,
    >,
    selected: &runtime::ProductBranchIdentity,
) -> domain::WorthQueryBoundDomainOperation<
    installed_operation_fixture::GeometryDomain,
    installed_operation_fixture::ConditionalResourceOperation,
    installed_operation_fixture::ConditionalResourceFamily,
    foundation::ObservationLaneWitness,
> {
    workspace
        .observe_product_operating_world(selected)
        .unwrap()
        .family(installed_operation_fixture::ConditionalResourceFamily)
        .bind(
            installed,
            installed_operation_fixture::ConditionalResourceOperation,
        )
        .unwrap()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConditionalExecutionObservation {
    slot_reuse_hits: usize,
    source_admission_attempts: usize,
    compute_contacts: usize,
}

fn execute_bound(
    workspace: &mut runtime::WorthQueryWorkspace,
    bound: domain::WorthQueryBoundDomainOperation<
        installed_operation_fixture::GeometryDomain,
        installed_operation_fixture::ConditionalResourceOperation,
        installed_operation_fixture::ConditionalResourceFamily,
        foundation::ObservationLaneWitness,
    >,
) -> ConditionalExecutionObservation {
    let outcome = bound
        .admit_execution_resources(
            (),
            installed_operation_fixture::execution_resource_request(),
            workspace,
        )
        .unwrap()
        .execute(workspace);
    match outcome {
        TransitionOutcome::Success(executed) => observation(executed.conditional_provenance()),
        TransitionOutcome::Deferred(deferred) => observation(deferred.conditional_provenance()),
        TransitionOutcome::Denied(denial) | TransitionOutcome::Failed(denial) => {
            panic!("selected conditional execution failed: {denial:?}")
        }
        _ => panic!("selected conditional execution did not complete"),
    }
}

fn observation(
    provenance: &[domain::WorthQueryConditionalProvenance],
) -> ConditionalExecutionObservation {
    let [conditional] = provenance else {
        panic!("the resource operation must execute one conditional node")
    };
    ConditionalExecutionObservation {
        slot_reuse_hits: conditional.signal_slot_reuse_hits(),
        source_admission_attempts: conditional.source_admission_attempts(),
        compute_contacts: conditional.compute_contacts(),
    }
}
