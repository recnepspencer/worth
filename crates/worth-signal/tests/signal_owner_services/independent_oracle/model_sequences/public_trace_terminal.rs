use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, ManagedSignalBranchReference, SignalBranchBasisPort,
    SignalBranchLifecyclePort, SignalBranchRetentionReleaseOutcome, SignalBranchRetirementDenial,
    SignalBranchRetirementReason, SignalOwnerCancellationSource, SignalOwnerLifecycleObservation,
};
use worth_signal::facade::SignalRuntime;

use super::super::comparison::{basis_observation_denial, neutral_basis, release_denial};
use super::super::state::{ModelOwnerLifecycle, ModelWorld};
use super::super::transition::{ModelAction, ModelDenial, ModelResult, ModelSuccess};
use super::trace_support::{assert_denial, current_model, model_denial, model_lease, ORACLE_SEED};

type Runtime = SignalRuntime<(), (), (), (), ()>;

pub(super) fn finish_terminal_trace(
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
        model.apply(ModelAction::Retain {
            branch: child,
            observation: neutral_basis(&child_basis),
        }),
        ModelResult::Success(ModelSuccess::Lease)
    ));
    let child_lease_key = model_lease(model, "child retain");
    let expected = model_denial(
        model.apply(ModelAction::Retire {
            branch: child,
            expected: neutral_basis(&child_basis),
            cancelled: false,
        }),
        "retained child retirement",
    );
    let denied_plan = lifecycle.plan_retirement_exact(
        child_basis.clone(),
        SignalBranchRetirementReason::Superseded,
    );
    let actual = match denied_plan {
        TransitionOutcome::Denied(denial) => match denial {
            SignalBranchRetirementDenial::RetainedComponentBasis { .. }
            | SignalBranchRetirementDenial::RetainedAdmittedBasis { .. } => {
                ModelDenial::RetainedBasis
            }
            other => panic!("seed {ORACLE_SEED:#x}: wrong child-retirement denial: {other:?}"),
        },
        other => panic!(
            "seed {ORACLE_SEED:#x}: retained child retirement unexpectedly planned: {other:?}"
        ),
    };
    assert_denial(expected, actual, "retained child retirement");
    match basis.release_exact(child_lease) {
        SignalBranchRetentionReleaseOutcome::Released(_) => {}
        other => panic!("seed {ORACLE_SEED:#x}: child lease release denied: {other:?}"),
    }
    assert!(matches!(
        model.apply(ModelAction::Release {
            lease: child_lease_key,
        }),
        ModelResult::Success(ModelSuccess::Release)
    ));

    let plan = lifecycle
        .plan_retirement_exact(child_basis, SignalBranchRetirementReason::Superseded)
        .into_result()
        .expect("released child is now exactly retireable");
    assert!(matches!(
        model.apply(ModelAction::Retire {
            branch: child,
            expected: current_model(model, child),
            cancelled: false,
        }),
        ModelResult::Success(ModelSuccess::Retirement)
    ));
    assert!(matches!(
        lifecycle.retire_exact(plan, &SignalOwnerCancellationSource::new().token()),
        TransitionOutcome::Success(_)
    ));
    let child_denial = basis
        .observe_current(child_reference)
        .expect_err("retired child observation must be a typed denial");
    assert!(matches!(
        model.apply(ModelAction::Observe { branch: child }),
        ModelResult::Denied(ModelDenial::RetiredBranch)
    ));
    assert!(matches!(
        basis_observation_denial(&child_denial),
        ModelDenial::RetiredBranch
    ));

    let close_lease = basis
        .retain_exact(&current)
        .expect("the final root lease remains live before owner loss");
    assert!(matches!(
        model.apply(ModelAction::Retain {
            branch: root,
            observation: current_model(model, root),
        }),
        ModelResult::Success(ModelSuccess::Lease)
    ));
    let close_lease_key = model_lease(model, "close lease");
    drop(runtime);
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    model.apply(ModelAction::Close);
    assert_eq!(
        model.lifecycle_history,
        vec![
            ModelOwnerLifecycle::Open,
            ModelOwnerLifecycle::Closing,
            ModelOwnerLifecycle::Closed,
        ]
    );
    let unavailable = basis
        .observe_current(reference)
        .expect_err("weak owner loss must deny root observation");
    assert!(matches!(
        basis_observation_denial(&unavailable),
        ModelDenial::OwnerUnavailable
    ));
    let denied_release = basis.release_exact(close_lease);
    let returned_lease = match denied_release {
        SignalBranchRetentionReleaseOutcome::Denied { lease, denial } => {
            assert_denial(
                model_denial(
                    model.apply(ModelAction::Release {
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
