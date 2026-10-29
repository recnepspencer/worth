pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{domain, foundation, product, runtime};
pub use worth_signal::facade::runtime::SignalConditionalEvaluationBudget;

pub use super::{conditional_node_contract, installed_operation_fixture};
pub use crate::facade::query::support::public_bridge_runtime::PublicBridgeRuntimeHarness;

pub fn create_sibling(
    workspace: &runtime::WorthQueryWorkspace,
    _name: &str,
    _ordinal: usize,
) -> product::WorthQueryProductBranch {
    workspace
        .with_advancement(|phase| {
            workspace
                .branches()
                .fork(workspace.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .expect("installed host call opens")
        .expect("World must publish the selected sibling")
}

pub fn execute_default(
    workspace: &mut runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<
        installed_operation_fixture::GeometryDomain,
    >,
) -> ConditionalExecutionObservation {
    let bound = bind_default(workspace, installed);
    execute_bound(workspace, bound)
}

pub fn execute_selected(
    workspace: &mut runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<
        installed_operation_fixture::GeometryDomain,
    >,
    selected: product::WorthQueryProductBranch,
) -> ConditionalExecutionObservation {
    let bound = bind_selected(workspace, installed, selected);
    execute_bound(workspace, bound)
}

pub fn bind_default(
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
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(installed_operation_fixture::ConditionalResourceFamily)
        .bind(
            installed,
            installed_operation_fixture::ConditionalResourceOperation,
        )
        .unwrap()
}

pub fn bind_selected(
    workspace: &runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<
        installed_operation_fixture::GeometryDomain,
    >,
    selected: product::WorthQueryProductBranch,
) -> domain::WorthQueryBoundDomainOperation<
    installed_operation_fixture::GeometryDomain,
    installed_operation_fixture::ConditionalResourceOperation,
    installed_operation_fixture::ConditionalResourceFamily,
    foundation::ObservationLaneWitness,
> {
    workspace
        .observe_operating_world(selected)
        .unwrap()
        .family(installed_operation_fixture::ConditionalResourceFamily)
        .bind(
            installed,
            installed_operation_fixture::ConditionalResourceOperation,
        )
        .unwrap()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConditionalExecutionObservation {
    pub slot_reuse_hits: usize,
    pub source_admission_attempts: usize,
    pub compute_contacts: usize,
}

pub fn execute_bound(
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

pub fn observation(
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
