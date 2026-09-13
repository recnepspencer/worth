use super::super::comparison::neutral_basis;
use super::super::state::{ModelOwnerLifecycle, ModelWorld};
use super::super::transition::{ModelAction, ModelDenial, ModelResult, ModelSuccess};
use super::trace_support::{assert_denial, model_denial, runtime, ORACLE_SEED};
use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    SignalBranchRetirementDenial, SignalBranchRetirementReason, SignalOwnerLifecycleObservation,
};

#[test]
fn model_preserves_current_branch_denial_instead_of_treating_it_as_retirement() {
    let mut runtime = runtime();
    let branch = runtime.current_branch();
    let basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("seed setup: current branch basis is admitted");
    let observation = neutral_basis(&basis);
    let mut model = ModelWorld::bootstrap(branch.id.0, branch.name.clone(), observation.clone());
    let services = runtime
        .owner_component_services()
        .expect("seed setup: service facade is issuable");
    let lifecycle = services.lifecycle_port();
    let expected = model_denial(
        model.apply(ModelAction::Retire {
            branch: branch.id.0,
            expected: observation,
            cancelled: false,
        }),
        "current-branch retirement",
    );
    let denial = services
        .lifecycle_port()
        .plan_retirement_exact(basis, SignalBranchRetirementReason::Rejected);
    let actual = match denial {
        TransitionOutcome::Denied(SignalBranchRetirementDenial::CurrentBranch { branch_id }) => {
            assert_eq!(branch_id, branch.id);
            ModelDenial::CurrentBranch
        }
        other => panic!(
            "seed {ORACLE_SEED:#x}: current branch changed lifecycle unexpectedly: {other:?}"
        ),
    };
    assert_denial(expected, actual, "current-branch retirement");
    drop(runtime);
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert_eq!(model.lifecycle_history.len(), 1);
    assert_eq!(model.lifecycle, ModelOwnerLifecycle::Open);
    assert!(matches!(
        model.apply(ModelAction::CapabilityLoss),
        ModelResult::Success(ModelSuccess::Closed)
    ));
}
