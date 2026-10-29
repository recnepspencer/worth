pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::branch::AdmittedSignalBranchBasis, worth_signal::facade::branch::ManagedSignalBranchReference, worth_signal::facade::branch::SignalBranchBasisPort, worth_signal::facade::branch::SignalBranchLifecyclePort, worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome, worth_signal::facade::branch::SignalBranchRetirementDenial, worth_signal::facade::branch::SignalBranchRetirementReason, worth_signal::facade::branch::SignalOwnerCancellationSource, worth_signal::facade::branch::SignalOwnerLifecycleObservation};
pub(crate) use {worth_signal::facade::SignalRuntime};

pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::comparison::basis_observation_denial, crate::signal_owner_services::reference_context::independent_oracle::comparison::neutral_basis, crate::signal_owner_services::reference_context::independent_oracle::comparison::release_denial};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle, crate::signal_owner_services::reference_context::independent_oracle::state::ModelWorld};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::assert_denial, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::current_model, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::model_denial, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::model_lease, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::ORACLE_SEED};

pub(crate) type Runtime = SignalRuntime<(), (), (), (), ()>;

pub(crate) fn finish_terminal_trace(
    runtime: Runtime,
    basis: &SignalBranchBasisPort<(), (), ()>,
    lifecycle: &SignalBranchLifecyclePort<(), (), ()>,
    model: &mut ModelWorld,
    root: u64,
    child: u64,
    current: AdmittedSignalBranchBasis,
    child_basis: AdmittedSignalBranchBasis,
    reference: &ManagedSignalBranchReference,
    child_reference: &ManagedSignalBranchReference,
) {
    let child_lease = basis
        .retain_exact(&child_basis)
        .expect("child retention opens before retirement planning");
    assert!(matches!(
        model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retain {
            branch: child,
            observation: neutral_basis(&child_basis),
        }),
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Lease)
    ));
    let child_lease_key = model_lease(model, "child retain");
    let expected = model_denial(
        model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retire {
            branch: child,
            expected: neutral_basis(&child_basis),
            cancelled: false,
        }),
        "retained child retirement",
    );
    let denied_plan = lifecycle.plan_retirement_exact(
        child_basis.clone(),
        worth_signal::facade::branch::SignalBranchRetirementReason::Superseded,
    );
    let actual = match denied_plan {
        worth_proof::TransitionOutcome::Denied(denial) => match denial {
            worth_signal::facade::branch::SignalBranchRetirementDenial::RetainedComponentBasis { .. }
            | worth_signal::facade::branch::SignalBranchRetirementDenial::RetainedAdmittedBasis { .. } => {
                crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetainedBasis
            }
            other => panic!("seed {ORACLE_SEED:#x}: wrong child-retirement denial: {other:?}"),
        },
        other => panic!(
            "seed {ORACLE_SEED:#x}: retained child retirement unexpectedly planned: {other:?}"
        ),
    };
    assert_denial(expected, actual, "retained child retirement");
    match basis.release_exact(child_lease) {
        worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Released(_) => {}
        other => panic!("seed {ORACLE_SEED:#x}: child lease release denied: {other:?}"),
    }
    assert!(matches!(
        model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Release {
            lease: child_lease_key,
        }),
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Release)
    ));

    let plan = lifecycle
        .plan_retirement_exact(child_basis, worth_signal::facade::branch::SignalBranchRetirementReason::Superseded)
        .into_result()
        .expect("released child is now exactly retireable");
    assert!(matches!(
        model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retire {
            branch: child,
            expected: current_model(model, child),
            cancelled: false,
        }),
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Retirement)
    ));
    assert!(matches!(
        lifecycle.retire_exact(plan, &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token()),
        worth_proof::TransitionOutcome::Success(_)
    ));
    let child_denial = basis
        .observe_current(child_reference)
        .expect_err("retired child observation must be a typed denial");
    assert!(matches!(
        model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Observe { branch: child }),
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch)
    ));
    assert!(matches!(
        basis_observation_denial(&child_denial),
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch
    ));

    let close_lease = basis
        .retain_exact(&current)
        .expect("the final root lease remains live before owner loss");
    assert!(matches!(
        model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retain {
            branch: root,
            observation: current_model(model, root),
        }),
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Lease)
    ));
    let close_lease_key = model_lease(model, "close lease");
    drop(runtime);
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        worth_signal::facade::branch::SignalOwnerLifecycleObservation::Closed
    );
    model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Close);
    assert_eq!(
        model.lifecycle_history,
        vec![
            crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle::Open,
            crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle::Closing,
            crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle::Closed,
        ]
    );
    let unavailable = basis
        .observe_current(reference)
        .expect_err("weak owner loss must deny root observation");
    assert!(matches!(
        basis_observation_denial(&unavailable),
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable
    ));
    let denied_release = basis.release_exact(close_lease);
    let returned_lease = match denied_release {
        worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Denied { lease, denial } => {
            assert_denial(
                model_denial(
                    model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Release {
                        lease: close_lease_key,
                    }),
                    "release after owner loss",
                ),
                release_denial(&denial),
                "release after owner loss",
            );
            lease
        }
        other => {
            panic!("seed {ORACLE_SEED:#x}: owner-loss release unexpectedly succeeded: {other:?}")
        }
    };
    drop(returned_lease);
}