pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome};
pub(crate) use {worth_signal::facade::branch::validate_signal_branch_name, worth_signal::facade::branch::SignalBranchRetirementReason, worth_signal::facade::branch::SignalOwnerCancellationSource};

pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::comparison::advance_denial, crate::signal_owner_services::reference_context::independent_oracle::comparison::capture_denial, crate::signal_owner_services::reference_context::independent_oracle::comparison::fork_denial, crate::signal_owner_services::reference_context::independent_oracle::comparison::neutral_basis, crate::signal_owner_services::reference_context::independent_oracle::comparison::neutral_snapshot, crate::signal_owner_services::reference_context::independent_oracle::comparison::release_denial, crate::signal_owner_services::reference_context::independent_oracle::comparison::restore_denial, crate::signal_owner_services::reference_context::independent_oracle::comparison::retention_denial};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess, crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::court::BranchSlot, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::court::HeldLease, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::court::PairCourt};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::retirement_denial, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess};

pub(crate) fn perform_real(
    court: &mut PairCourt,
    operation: OperationKind,
    step: usize,
) -> RealResult {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    match operation {
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Fork => {
            let name = court.child_name(step);
            let source = court.root.basis.clone();
            let identity = validate_signal_branch_name(&name)
                .expect("adjacency operation identity is generated from validated indices");
            match court.mutation.fork_exact(
                identity,
                &source,
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            ) {
                Ok(outcome) => {
                    let (branch, basis) = outcome.into_parts();
                    let child_id = branch.id.0;
                    crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Fork {
                        observation: neutral_basis(&basis),
                        child_id,
                        basis,
                    })
                }
                Err(denial) => crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(fork_denial(&denial)),
            }
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Advance => {
            let expected = court.root.basis.clone();
            match court.mutation.advance_exact(
                request_execution,
                &expected,
                &mut (),
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
                |_| Ok(()),
            ) {
                Ok(outcome) => {
                    let basis = outcome.into_basis();
                    crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Advance {
                        observation: neutral_basis(&basis),
                        basis,
                    })
                }
                Err(denial) => crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(advance_denial(&denial)),
            }
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Capture => {
            let expected = court.root.basis.clone();
            match court
                .mutation
                .capture_exact(&expected, &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token())
            {
                Ok(outcome) => {
                    let (admitted_snapshot, basis) = outcome.into_parts();
                    let snapshot = neutral_snapshot(&admitted_snapshot);
                    crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Capture {
                        observation: neutral_basis(&basis),
                        snapshot,
                        admitted_snapshot: Box::new(admitted_snapshot),
                        basis,
                    })
                }
                Err(denial) => crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(capture_denial(&denial)),
            }
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Restore => {
            let expected = court.root.basis.clone();
            let snapshot = court
                .snapshots
                .last()
                .map(|(_, snapshot)| snapshot.clone())
                .expect("adjacency setup installs a restore snapshot");
            match court.mutation.restore_exact(
                &expected,
                &snapshot,
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            ) {
                Ok(basis) => crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Restore {
                    observation: neutral_basis(&basis),
                    basis,
                }),
                Err(denial) => crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(restore_denial(&denial)),
            }
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Retain => {
            let expected = court.root.basis.clone();
            match court.basis.retain_exact(&expected) {
                Ok(lease) => crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Lease(lease)),
                Err(denial) => crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(retention_denial(&denial)),
            }
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Release => {
            let held = court
                .leases
                .pop()
                .expect("adjacency setup has two leases for two release positions");
            let model_key = held.model_key;
            match court.basis.release_exact(held.lease) {
                worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Released(_) => {
                    crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Release { model_key })
                }
                worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome::Denied { lease, denial } => {
                    court.leases.push(HeldLease { model_key, lease });
                    crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(release_denial(&denial))
                }
            }
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Retire => {
            let target = court
                .retirement_target
                .take()
                .expect("adjacency court retains a retirement target");
            let plan = court
                .lifecycle
                .plan_retirement_exact(target.basis, worth_signal::facade::branch::SignalBranchRetirementReason::Superseded);
            match plan {
                worth_proof::TransitionOutcome::Denied(denial) => crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(retirement_denial(&denial)),
                worth_proof::TransitionOutcome::Success(plan) => match court
                    .lifecycle
                    .retire_exact(plan, &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token())
                {
                    worth_proof::TransitionOutcome::Success(_) => crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Retirement),
                    worth_proof::TransitionOutcome::Denied(denial) => {
                        crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(retirement_denial(&denial))
                    }
                },
            }
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Close | crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::CapabilityLoss => {
            if court.drop_owner() {
                crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Closed)
            } else {
                crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable)
            }
        }
    }
}

pub(crate) fn model_action(
    court: &PairCourt,
    operation: OperationKind,
    real: &RealResult,
    step: usize,
) -> ModelAction {
    match operation {
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Fork => {
            let child = match real {
                crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Fork { child_id, .. }) => *child_id,
                _ => court.next_child_id(),
            };
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Fork {
                source: court.root.id,
                child,
                child_name: court.child_name(step),
            }
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Advance => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Advance {
            branch: court.root.id,
            expected: court.current_root_model(),
            cancelled: false,
        },
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Capture => {
            let snapshot = match real {
                crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Capture { snapshot, .. }) => snapshot.snapshot,
                _ => 0,
            };
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Capture {
                branch: court.root.id,
                expected: court.current_root_model(),
                snapshot,
                cancelled: false,
            }
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Restore => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Restore {
            branch: court.root.id,
            expected: court.current_root_model(),
            snapshot: court.latest_snapshot(),
            cancelled: false,
        },
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Retain => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retain {
            branch: court.root.id,
            observation: court.current_root_model(),
        },
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Release => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Release {
            lease: match real {
                crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Release { model_key }) => *model_key,
                _ => court.release_key(),
            },
        },
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Retire => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retire {
            branch: court.retirement_target_id,
            expected: court.target_model(),
            cancelled: false,
        },
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Close => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Close,
        crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::CapabilityLoss => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::CapabilityLoss,
    }
}

pub(crate) fn commit(
    court: &mut PairCourt,
    operation: OperationKind,
    real: RealResult,
    expected: &ModelResult,
) {
    match (operation, real, expected) {
        (
            crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Fork,
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Fork {
                child_id, basis, ..
            }),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Fork(_)),
        ) => {
            court.retirement_target = Some(BranchSlot {
                id: child_id,
                basis,
            });
            court.retirement_target_id = child_id;
        }
        (
            crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Advance,
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Advance { basis, .. }),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Advance(_)),
        )
        | (
            crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Restore,
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Restore { basis, .. }),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Restore(_)),
        ) => {
            court.root.basis = basis;
        }
        (
            crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Capture,
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Capture {
                basis,
                snapshot,
                admitted_snapshot,
                ..
            }),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Capture { .. }),
        ) => {
            court.root.basis = basis;
            court.snapshots.push((snapshot, *admitted_snapshot));
        }
        (
            crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Retain,
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Lease(lease)),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Lease),
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
            crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Retire,
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Retirement),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Retirement),
        ) => {
            court.retirement_target = court.secondary_retirement_target.take();
            if let Some(target) = &court.retirement_target {
                court.retirement_target_id = target.id;
            }
        }
        _ => {}
    }
}