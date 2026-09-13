use worth_signal::facade::branch::{
    validate_signal_branch_name, AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot,
    SignalBranchBasisPort, SignalBranchLifecyclePort, SignalBranchMutationPort,
    SignalBranchRetentionLease, SignalOwnerCancellationSource,
};
use worth_signal::facade::SignalRuntime;

use super::super::super::comparison::{neutral_basis, neutral_snapshot};
use super::super::super::state::{ModelObservation, ModelSnapshot, ModelWorld};
use super::super::super::transition::{ModelAction, OperationKind};
use super::super::trace_support::{model_lease, runtime, ORACLE_SEED};
use super::outcome::{assert_equivalent, RealResult, RealSuccess};

pub(super) type Runtime = SignalRuntime<(), (), (), (), ()>;
pub(super) type BasisPort = SignalBranchBasisPort<(), (), ()>;
pub(super) type MutationPort = SignalBranchMutationPort<(), (), (), (), ()>;
pub(super) type LifecyclePort = SignalBranchLifecyclePort<(), (), ()>;

#[derive(Clone)]
pub(super) struct BranchSlot {
    pub(super) id: u64,
    pub(super) basis: AdmittedSignalBranchBasis,
}

pub(super) struct HeldLease {
    pub(super) model_key: u64,
    pub(super) lease: SignalBranchRetentionLease,
}

pub(super) struct PairCourt {
    pub(super) runtime: Option<Runtime>,
    pub(super) basis: BasisPort,
    pub(super) mutation: MutationPort,
    pub(super) lifecycle: LifecyclePort,
    pub(super) root: BranchSlot,
    pub(super) retirement_target: Option<BranchSlot>,
    pub(super) secondary_retirement_target: Option<BranchSlot>,
    pub(super) retirement_target_id: u64,
    pub(super) snapshots: Vec<(ModelSnapshot, AdmittedSignalBranchSnapshot)>,
    pub(super) leases: Vec<HeldLease>,
    pub(super) model: ModelWorld,
    pub(super) pair_index: (usize, usize),
}

impl PairCourt {
    pub(super) fn new(first_index: usize, second_index: usize) -> Self {
        let mut runtime = runtime();
        let root_handle = runtime.current_branch();
        let root_basis = runtime
            .observe_signal_branch_basis(root_handle.clone())
            .expect("adjacency setup: bootstrap basis is admitted");
        let root = root_handle.id.0;
        let root_observation = neutral_basis(&root_basis);
        let mut model =
            ModelWorld::bootstrap(root, root_handle.name.clone(), root_observation.clone());

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
                &SignalOwnerCancellationSource::new().token(),
            )
            .expect("adjacency setup: real owner creates the retirement target")
            .into_parts();
        let child = child_handle.id.0;
        let setup_fork = RealResult::Success(RealSuccess::Fork {
            observation: neutral_basis(&child_basis),
            child_id: child,
            basis: child_basis.clone(),
        });
        let expected = model.apply(ModelAction::Fork {
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
                &SignalOwnerCancellationSource::new().token(),
            )
            .expect("adjacency setup: real owner creates a second retirement target")
            .into_parts();
        let secondary = secondary_handle.id.0;
        let setup_fork = RealResult::Success(RealSuccess::Fork {
            observation: neutral_basis(&secondary_basis),
            child_id: secondary,
            basis: secondary_basis.clone(),
        });
        let expected = model.apply(ModelAction::Fork {
            source: root,
            child: secondary,
            child_name: secondary_name.to_owned(),
        });
        assert_equivalent(&expected, &setup_fork, "adjacency setup secondary fork");
        drop(setup_fork);
        let capture = mutation
            .capture_exact(&root_basis, &SignalOwnerCancellationSource::new().token())
            .expect("adjacency setup: real owner captures the root");
        let (admitted_snapshot, captured_basis) = capture.into_parts();
        let snapshot = neutral_snapshot(&admitted_snapshot);
        let captured_observation = neutral_basis(&captured_basis);
        let expected = model.apply(ModelAction::Capture {
            branch: root,
            expected: root_observation,
            snapshot: snapshot.snapshot,
            cancelled: false,
        });
        let setup_capture = RealResult::Success(RealSuccess::Capture {
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
            let expected = model.apply(ModelAction::Retain {
                branch: root,
                observation: neutral_basis(&current_root),
            });
            let key = model_lease(&model, "adjacency setup retain");
            let setup_lease = RealResult::Success(RealSuccess::Lease(lease));
            assert_equivalent(&expected, &setup_lease, "adjacency setup retain");
            let lease = match setup_lease {
                RealResult::Success(RealSuccess::Lease(lease)) => lease,
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

    pub(super) fn apply_pair_operation(&mut self, operation: OperationKind, step: usize) {
        let real = super::operation::perform_real(self, operation, step);
        let action = super::operation::model_action(self, operation, &real, step);
        let expected = self.model.apply(action);
        let context = self.context(operation, step);
        assert_equivalent(&expected, &real, &context);
        super::operation::commit(self, operation, real, &expected);
    }

    pub(super) fn context(&self, operation: OperationKind, step: usize) -> String {
        format!(
            "seed {ORACLE_SEED:#x}, pair {}->{}, step {step} {operation:?}",
            self.pair_index.0, self.pair_index.1
        )
    }

    pub(super) fn child_name(&self, step: usize) -> String {
        format!(
            "oracle-adjacency-{}-{}-{step}",
            self.pair_index.0, self.pair_index.1
        )
    }

    pub(super) fn next_child_id(&self) -> u64 {
        self.model
            .branches
            .keys()
            .next_back()
            .copied()
            .unwrap_or(self.root.id)
            .saturating_add(1)
    }

    pub(super) fn current_root_model(&self) -> ModelObservation {
        self.model
            .branch(self.root.id)
            .expect("adjacency oracle retains its root branch")
            .observation
            .clone()
    }

    pub(super) fn target_model(&self) -> ModelObservation {
        self.model
            .branch(self.retirement_target_id)
            .expect("adjacency oracle retains its retirement target")
            .observation
            .clone()
    }

    pub(super) fn latest_snapshot(&self) -> ModelSnapshot {
        self.snapshots
            .last()
            .map(|(snapshot, _)| snapshot.clone())
            .expect("adjacency setup always installs one restore snapshot")
    }

    pub(super) fn release_key(&self) -> u64 {
        self.leases
            .last()
            .map(|lease| lease.model_key)
            .expect("adjacency setup holds two leases for the two-step roster")
    }

    pub(super) fn drop_owner(&mut self) -> bool {
        self.runtime.take().is_some()
    }
}
