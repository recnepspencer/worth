pub(crate) use {worth_signal::facade::branch::validate_signal_branch_name, worth_signal::facade::branch::AdmittedSignalBranchBasis, worth_signal::facade::branch::AdmittedSignalBranchSnapshot, worth_signal::facade::branch::SignalBranchBasisPort, worth_signal::facade::branch::SignalBranchLifecyclePort, worth_signal::facade::branch::SignalBranchMutationPort, worth_signal::facade::branch::SignalBranchRetentionLease, worth_signal::facade::branch::SignalOwnerCancellationSource};
pub(crate) use {worth_signal::facade::SignalRuntime};

pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::comparison::neutral_basis, crate::signal_owner_services::reference_context::independent_oracle::comparison::neutral_snapshot};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::state::ModelObservation, crate::signal_owner_services::reference_context::independent_oracle::state::ModelSnapshot, crate::signal_owner_services::reference_context::independent_oracle::state::ModelWorld};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction, crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::model_lease, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::runtime, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::trace_support::ORACLE_SEED};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::assert_equivalent, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult, crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess};

pub(crate) type Runtime = SignalRuntime<(), (), (), (), ()>;
pub(crate) type BasisPort = SignalBranchBasisPort<(), (), ()>;
pub(crate) type MutationPort = SignalBranchMutationPort<(), (), (), (), ()>;
pub(crate) type LifecyclePort = SignalBranchLifecyclePort<(), (), ()>;

#[derive(Clone)]
pub(crate) struct BranchSlot {
    pub(crate) id: u64,
    pub(crate) basis: AdmittedSignalBranchBasis,
}

pub(crate) struct HeldLease {
    pub(crate) model_key: u64,
    pub(crate) lease: SignalBranchRetentionLease,
}

pub(crate) struct PairCourt {
    pub(crate) runtime: Option<Runtime>,
    pub(crate) basis: BasisPort,
    pub(crate) mutation: MutationPort,
    pub(crate) lifecycle: LifecyclePort,
    pub(crate) root: BranchSlot,
    pub(crate) retirement_target: Option<BranchSlot>,
    pub(crate) secondary_retirement_target: Option<BranchSlot>,
    pub(crate) retirement_target_id: u64,
    pub(crate) snapshots: Vec<(ModelSnapshot, AdmittedSignalBranchSnapshot)>,
    pub(crate) leases: Vec<HeldLease>,
    pub(crate) model: ModelWorld,
    pub(crate) pair_index: (usize, usize),
}

impl PairCourt {
    pub(crate) fn new(first_index: usize, second_index: usize) -> Self {
        let mut runtime = runtime();
        let root_handle = runtime.current_branch();
        let root_basis = runtime
            .observe_signal_branch_basis(root_handle.clone())
            .expect("adjacency setup: bootstrap basis is admitted");
        let root = root_handle.id.0;
        let root_observation = neutral_basis(&root_basis);
        let mut model =
            crate::signal_owner_services::reference_context::independent_oracle::state::ModelWorld::bootstrap(root, root_handle.name.clone(), root_observation.clone());

        let services = runtime
            .owner_component_services()
            .expect("adjacency setup: service facade is issuable");
        let basis = services.basis_port();
        let mutation = services.mutation_port();
        let lifecycle = services.lifecycle_port();
        let child_name = "oracle-adjacency-setup-child";
        let (child_handle, child_basis) = mutation
            .fork_exact(
                validate_signal_branch_name(child_name)
                    .expect("adjacency setup: child identity validates"),
                &root_basis,
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            )
            .expect("adjacency setup: real owner creates the retirement target")
            .into_parts();
        let child = child_handle.id.0;
        let setup_fork = crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Fork {
            observation: neutral_basis(&child_basis),
            child_id: child,
            basis: child_basis.clone(),
        });
        let expected = model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Fork {
            source: root,
            child,
            child_name: child_name.to_owned(),
        });
        assert_equivalent(&expected, &setup_fork, "adjacency setup fork");
        drop(setup_fork);

        let secondary_name = "oracle-adjacency-setup-child-b";
        let (secondary_handle, secondary_basis) = mutation
            .fork_exact(
                validate_signal_branch_name(secondary_name)
                    .expect("adjacency setup: secondary identity validates"),
                &root_basis,
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            )
            .expect("adjacency setup: real owner creates a second retirement target")
            .into_parts();
        let secondary = secondary_handle.id.0;
        let setup_fork = crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Fork {
            observation: neutral_basis(&secondary_basis),
            child_id: secondary,
            basis: secondary_basis.clone(),
        });
        let expected = model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Fork {
            source: root,
            child: secondary,
            child_name: secondary_name.to_owned(),
        });
        assert_equivalent(&expected, &setup_fork, "adjacency setup secondary fork");
        drop(setup_fork);
        let capture = mutation
            .capture_exact(&root_basis, &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token())
            .expect("adjacency setup: real owner captures the root");
        let (admitted_snapshot, captured_basis) = capture.into_parts();
        let snapshot = neutral_snapshot(&admitted_snapshot);
        let captured_observation = neutral_basis(&captured_basis);
        let expected = model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Capture {
            branch: root,
            expected: root_observation,
            snapshot: snapshot.snapshot,
            cancelled: false,
        });
        let setup_capture = crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Capture {
            observation: captured_observation,
            snapshot: snapshot.clone(),
            admitted_snapshot: Box::new(admitted_snapshot.clone()),
            basis: captured_basis.clone(),
        });
        assert_equivalent(&expected, &setup_capture, "adjacency setup capture");
        drop(setup_capture);

        let mut leases = Vec::new();
        let mut current_root = captured_basis;
        for ordinal in 0..2 {
            let lease = basis.retain_exact(&current_root).unwrap_or_else(|denial| {
                panic!("seed {ORACLE_SEED:#x}: adjacency setup retain {ordinal} denied: {denial:?}")
            });
            let expected = model.apply(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retain {
                branch: root,
                observation: neutral_basis(&current_root),
            });
            let key = model_lease(&model, "adjacency setup retain");
            let setup_lease = crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Lease(lease));
            assert_equivalent(&expected, &setup_lease, "adjacency setup retain");
            let lease = match setup_lease {
                crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Lease(lease)) => lease,
                _ => unreachable!("setup lease is always a success"),
            };
            leases.push(HeldLease {
                model_key: key,
                lease,
            });
            current_root = current_root.clone();
        }

        Self {
            runtime: Some(runtime),
            basis,
            mutation,
            lifecycle,
            root: BranchSlot {
                id: root,
                basis: current_root,
            },
            retirement_target: Some(BranchSlot {
                id: child,
                basis: child_basis,
            }),
            secondary_retirement_target: Some(BranchSlot {
                id: secondary,
                basis: secondary_basis,
            }),
            retirement_target_id: child,
            snapshots: vec![(snapshot, admitted_snapshot)],
            leases,
            model,
            pair_index: (first_index, second_index),
        }
    }

    pub(crate) fn apply_pair_operation(&mut self, operation: OperationKind, step: usize) {
        let real = crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::operation::perform_real(self, operation, step);
        let action = crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::operation::model_action(self, operation, &real, step);
        let expected = self.model.apply(action);
        let context = self.context(operation, step);
        assert_equivalent(&expected, &real, &context);
        crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::operation::commit(self, operation, real, &expected);
    }

    pub(crate) fn context(&self, operation: OperationKind, step: usize) -> String {
        format!(
            "seed {ORACLE_SEED:#x}, pair {}->{}, step {step} {operation:?}",
            self.pair_index.0, self.pair_index.1
        )
    }

    pub(crate) fn child_name(&self, step: usize) -> String {
        format!(
            "oracle-adjacency-{}-{}-{step}",
            self.pair_index.0, self.pair_index.1
        )
    }

    pub(crate) fn next_child_id(&self) -> u64 {
        self.model
            .branches
            .keys()
            .next_back()
            .copied()
            .unwrap_or(self.root.id)
            .saturating_add(1)
    }

    pub(crate) fn current_root_model(&self) -> ModelObservation {
        self.model
            .branch(self.root.id)
            .expect("adjacency oracle retains its root branch")
            .observation
            .clone()
    }

    pub(crate) fn target_model(&self) -> ModelObservation {
        self.model
            .branch(self.retirement_target_id)
            .expect("adjacency oracle retains its retirement target")
            .observation
            .clone()
    }

    pub(crate) fn latest_snapshot(&self) -> ModelSnapshot {
        self.snapshots
            .last()
            .map(|(snapshot, _)| snapshot.clone())
            .expect("adjacency setup always installs one restore snapshot")
    }

    pub(crate) fn release_key(&self) -> u64 {
        self.leases
            .last()
            .map(|lease| lease.model_key)
            .expect("adjacency setup holds two leases for the two-step roster")
    }

    pub(crate) fn drop_owner(&mut self) -> bool {
        self.runtime.take().is_some()
    }
}