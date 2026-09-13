use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome;
use worth_signal::facade::branch::{
    validate_signal_branch_name, SignalBranchRetirementReason, SignalOwnerCancellationSource,
};

use super::super::super::comparison::{
    advance_denial, capture_denial, fork_denial, neutral_basis, neutral_snapshot, release_denial,
    restore_denial, retention_denial,
};
use super::super::super::transition::{
    ModelAction, ModelDenial, ModelResult, ModelSuccess, OperationKind,
};
use super::court::{BranchSlot, HeldLease, PairCourt};
use super::outcome::{retirement_denial, RealResult, RealSuccess};

pub(super) fn perform_real(
    court: &mut PairCourt,
    operation: OperationKind,
    step: usize,
) -> RealResult {
    match operation {
        OperationKind::Fork => {
            let name = court.child_name(step);
            let source = court.root.basis.clone();
            let identity = validate_signal_branch_name(&name)
                .expect("adjacency operation identity is generated from validated indices");
            match court.mutation.fork_exact(
                identity,
                &source,
                &SignalOwnerCancellationSource::new().token(),
            ) {
                Ok(outcome) => {
                    let (branch, basis) = outcome.into_parts();
                    let child_id = branch.id.0;
                    RealResult::Success(RealSuccess::Fork {
                        observation: neutral_basis(&basis),
                        child_id,
                        basis,
                    })
                }
                Err(denial) => RealResult::Denied(fork_denial(&denial)),
            }
        }
        OperationKind::Advance => {
            let expected = court.root.basis.clone();
            match court.mutation.advance_exact(
                &expected,
                &mut (),
                &SignalOwnerCancellationSource::new().token(),
                |_| Ok(()),
            ) {
                Ok(outcome) => {
                    let basis = outcome.into_basis();
                    RealResult::Success(RealSuccess::Advance {
                        observation: neutral_basis(&basis),
                        basis,
                    })
                }
                Err(denial) => RealResult::Denied(advance_denial(&denial)),
            }
        }
        OperationKind::Capture => {
            let expected = court.root.basis.clone();
            match court
                .mutation
                .capture_exact(&expected, &SignalOwnerCancellationSource::new().token())
            {
                Ok(outcome) => {
                    let (admitted_snapshot, basis) = outcome.into_parts();
                    let snapshot = neutral_snapshot(&admitted_snapshot);
                    RealResult::Success(RealSuccess::Capture {
                        observation: neutral_basis(&basis),
                        snapshot,
                        admitted_snapshot: Box::new(admitted_snapshot),
                        basis,
                    })
                }
                Err(denial) => RealResult::Denied(capture_denial(&denial)),
            }
        }
        OperationKind::Restore => {
            let expected = court.root.basis.clone();
            let snapshot = court
                .snapshots
                .last()
                .map(|(_, snapshot)| snapshot.clone())
                .expect("adjacency setup installs a restore snapshot");
            match court.mutation.restore_exact(
                &expected,
                &snapshot,
                &SignalOwnerCancellationSource::new().token(),
            ) {
                Ok(basis) => RealResult::Success(RealSuccess::Restore {
                    observation: neutral_basis(&basis),
                    basis,
                }),
                Err(denial) => RealResult::Denied(restore_denial(&denial)),
            }
        }
        OperationKind::Retain => {
            let expected = court.root.basis.clone();
            match court.basis.retain_exact(&expected) {
                Ok(lease) => RealResult::Success(RealSuccess::Lease(lease)),
                Err(denial) => RealResult::Denied(retention_denial(&denial)),
            }
        }
        OperationKind::Release => {
            let held = court
                .leases
                .pop()
                .expect("adjacency setup has two leases for two release positions");
            let model_key = held.model_key;
            match court.basis.release_exact(held.lease) {
                SignalBranchRetentionReleaseOutcome::Released(_) => {
                    RealResult::Success(RealSuccess::Release { model_key })
                }
                SignalBranchRetentionReleaseOutcome::Denied { lease, denial } => {
                    court.leases.push(HeldLease { model_key, lease });
                    RealResult::Denied(release_denial(&denial))
                }
            }
        }
        OperationKind::Retire => {
            let target = court
                .retirement_target
                .take()
                .expect("adjacency court retains a retirement target");
            let plan = court
                .lifecycle
                .plan_retirement_exact(target.basis, SignalBranchRetirementReason::Superseded);
            match plan {
                TransitionOutcome::Denied(denial) => RealResult::Denied(retirement_denial(&denial)),
                TransitionOutcome::Success(plan) => match court
                    .lifecycle
                    .retire_exact(plan, &SignalOwnerCancellationSource::new().token())
                {
                    TransitionOutcome::Success(_) => RealResult::Success(RealSuccess::Retirement),
                    TransitionOutcome::Denied(denial) => {
                        RealResult::Denied(retirement_denial(&denial))
                    }
                },
            }
        }
        OperationKind::Close | OperationKind::CapabilityLoss => {
            if court.drop_owner() {
                RealResult::Success(RealSuccess::Closed)
            } else {
                RealResult::Denied(ModelDenial::OwnerUnavailable)
            }
        }
    }
}

pub(super) fn model_action(
    court: &PairCourt,
    operation: OperationKind,
    real: &RealResult,
    step: usize,
) -> ModelAction {
    match operation {
        OperationKind::Fork => {
            let child = match real {
                RealResult::Success(RealSuccess::Fork { child_id, .. }) => *child_id,
                _ => court.next_child_id(),
            };
            ModelAction::Fork {
                source: court.root.id,
                child,
                child_name: court.child_name(step),
            }
        }
        OperationKind::Advance => ModelAction::Advance {
            branch: court.root.id,
            expected: court.current_root_model(),
            cancelled: false,
        },
        OperationKind::Capture => {
            let snapshot = match real {
                RealResult::Success(RealSuccess::Capture { snapshot, .. }) => snapshot.snapshot,
                _ => 0,
            };
            ModelAction::Capture {
                branch: court.root.id,
                expected: court.current_root_model(),
                snapshot,
                cancelled: false,
            }
        }
        OperationKind::Restore => ModelAction::Restore {
            branch: court.root.id,
            expected: court.current_root_model(),
            snapshot: court.latest_snapshot(),
            cancelled: false,
        },
        OperationKind::Retain => ModelAction::Retain {
            branch: court.root.id,
            observation: court.current_root_model(),
        },
        OperationKind::Release => ModelAction::Release {
            lease: match real {
                RealResult::Success(RealSuccess::Release { model_key }) => *model_key,
                _ => court.release_key(),
            },
        },
        OperationKind::Retire => ModelAction::Retire {
            branch: court.retirement_target_id,
            expected: court.target_model(),
            cancelled: false,
        },
        OperationKind::Close => ModelAction::Close,
        OperationKind::CapabilityLoss => ModelAction::CapabilityLoss,
    }
}

pub(super) fn commit(
    court: &mut PairCourt,
    operation: OperationKind,
    real: RealResult,
    expected: &ModelResult,
) {
    match (operation, real, expected) {
        (
            OperationKind::Fork,
            RealResult::Success(RealSuccess::Fork {
                child_id, basis, ..
            }),
            ModelResult::Success(ModelSuccess::Fork(_)),
        ) => {
            court.retirement_target = Some(BranchSlot {
                id: child_id,
                basis,
            });
            court.retirement_target_id = child_id;
        }
        (
            OperationKind::Advance,
            RealResult::Success(RealSuccess::Advance { basis, .. }),
            ModelResult::Success(ModelSuccess::Advance(_)),
        )
        | (
            OperationKind::Restore,
            RealResult::Success(RealSuccess::Restore { basis, .. }),
            ModelResult::Success(ModelSuccess::Restore(_)),
        ) => {
            court.root.basis = basis;
        }
        (
            OperationKind::Capture,
            RealResult::Success(RealSuccess::Capture {
                basis,
                snapshot,
                admitted_snapshot,
                ..
            }),
            ModelResult::Success(ModelSuccess::Capture { .. }),
        ) => {
            court.root.basis = basis;
            court.snapshots.push((snapshot, *admitted_snapshot));
        }
        (
            OperationKind::Retain,
            RealResult::Success(RealSuccess::Lease(lease)),
            ModelResult::Success(ModelSuccess::Lease),
        ) => {
            let model_key = court
                .model
                .leases
                .keys()
                .next_back()
                .copied()
                .expect("the model records every successful retain");
            court.leases.push(HeldLease { model_key, lease });
        }
        (
            OperationKind::Retire,
            RealResult::Success(RealSuccess::Retirement),
            ModelResult::Success(ModelSuccess::Retirement),
        ) => {
            court.retirement_target = court.secondary_retirement_target.take();
            if let Some(target) = &court.retirement_target {
                court.retirement_target_id = target.id;
            }
        }
        _ => {}
    }
}
