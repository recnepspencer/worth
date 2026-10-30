pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::comparison::neutral_basis};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle, crate::signal_owner_services::reference_context::independent_oracle::state::ModelWorld};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::assert_denial, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::model_denial, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::runtime, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::ORACLE_SEED};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::branch::SignalBranchRetirementDenial, worth_signal::facade::branch::SignalBranchRetirementReason, worth_signal::facade::branch::SignalOwnerLifecycleObservation};

#[test]
fn model_preserves_current_branch_denial_instead_of_treating_it_as_retirement() {
    let mut runtime = runtime();
    let branch = runtime.current_branch();
    let basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("seed setup: current branch basis is admitted");
    let observation = neutral_basis(&basis);
    let mut model = crate::signal_owner_services::reference_context::independent_oracle::state::ModelWorld::bootstrap(branch.id.0, branch.name.clone(), observation.clone());
    let services = runtime
        .owner_component_services()
        .expect("seed setup: service facade is issuable");
    let lifecycle = services.lifecycle_port();
    let expected = model_denial(
        model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retire {
            branch: branch.id.0,
            expected: observation,
            cancelled: false,
        }),
        "current-branch retirement",
    );
    let denial = services.lifecycle_port().plan_retirement_exact(
        basis,
        worth_signal::facade::branch::SignalBranchRetirementReason::Rejected,
    );
    let actual = match denial {
        worth_proof::TransitionOutcome::Denied(
            worth_signal::facade::branch::SignalBranchRetirementDenial::CurrentBranch { branch_id },
        ) => {
            assert_eq!(branch_id, branch.id);
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::CurrentBranch
        }
        other => panic!(
            "seed {ORACLE_SEED:#x}: current branch changed lifecycle unexpectedly: {other:?}"
        ),
    };
    assert_denial(expected, actual, "current-branch retirement");
    drop(runtime);
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        worth_signal::facade::branch::SignalOwnerLifecycleObservation::Closed
    );
    assert_eq!(model.lifecycle_history.len(), 1);
    assert_eq!(model.lifecycle, crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle::Open);
    assert!(matches!(
        model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::CapabilityLoss),
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Closed)
    ));
}
