#[path = "transition/mutations.rs"]
mod mutations;

use super::state::{
    BranchKey, ModelBranch, ModelBranchLifecycle, ModelObservation, ModelOwnerLifecycle,
    ModelSnapshot, ModelWorld,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum OperationKind {
    Retain,
    Release,
    Fork,
    Advance,
    Capture,
    Restore,
    Retire,
    Close,
    CapabilityLoss,
}

/// The model coverage contract is explicit so a shortened loop cannot look
/// green while omitting one operation or one ordered state-changing pair.
pub(crate) const STATE_CHANGING_OPERATIONS: [OperationKind; 9] = [
    OperationKind::Fork,
    OperationKind::Advance,
    OperationKind::Capture,
    OperationKind::Restore,
    OperationKind::Retain,
    OperationKind::Release,
    OperationKind::Retire,
    OperationKind::Close,
    OperationKind::CapabilityLoss,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelDenial {
    OwnerUnavailable,
    UnknownBranch,
    RetiredBranch,
    StaleBasis,
    Cancelled,
    RetainedBasis,
    ForeignSnapshot,
    CurrentBranch,
    MissingLease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelSuccess {
    Observation(ModelObservation),
    Fork(ModelObservation),
    Advance(ModelObservation),
    Capture {
        observation: ModelObservation,
        snapshot: ModelSnapshot,
    },
    Restore(ModelObservation),
    Lease,
    Release,
    Retirement,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelResult {
    Success(ModelSuccess),
    Denied(ModelDenial),
}

#[derive(Debug, Clone)]
pub(crate) enum ModelAction {
    Observe {
        branch: BranchKey,
    },
    Readmit {
        branch: BranchKey,
        expected: ModelObservation,
    },
    ReadmitRetained {
        branch: BranchKey,
        expected: ModelObservation,
        lease: u64,
    },
    Retain {
        branch: BranchKey,
        observation: ModelObservation,
    },
    Release {
        lease: u64,
    },
    Fork {
        source: BranchKey,
        child: BranchKey,
        child_name: String,
    },
    Advance {
        branch: BranchKey,
        expected: ModelObservation,
        cancelled: bool,
    },
    Capture {
        branch: BranchKey,
        expected: ModelObservation,
        snapshot: u64,
        cancelled: bool,
    },
    Restore {
        branch: BranchKey,
        expected: ModelObservation,
        snapshot: ModelSnapshot,
        cancelled: bool,
    },
    Retire {
        branch: BranchKey,
        expected: ModelObservation,
        cancelled: bool,
    },
    Close,
    CapabilityLoss,
}

impl ModelWorld {
    pub(crate) fn apply(&mut self, action: ModelAction) -> ModelResult {
        match action {
            ModelAction::Observe { branch } => self.observe(branch),
            ModelAction::Readmit { branch, expected } => self.readmit(branch, &expected),
            ModelAction::ReadmitRetained {
                branch,
                expected,
                lease,
            } => self.readmit_retained(branch, &expected, lease),
            ModelAction::Retain {
                branch,
                observation,
            } => self.retain(branch, observation),
            ModelAction::Release { lease } => self.release(lease),
            ModelAction::Fork {
                source,
                child,
                child_name,
            } => self.fork(source, child, child_name),
            ModelAction::Advance {
                branch,
                expected,
                cancelled,
            } => self.advance(branch, &expected, cancelled),
            ModelAction::Capture {
                branch,
                expected,
                snapshot,
                cancelled,
            } => self.capture(branch, &expected, snapshot, cancelled),
            ModelAction::Restore {
                branch,
                expected,
                snapshot,
                cancelled,
            } => self.restore(branch, &expected, &snapshot, cancelled),
            ModelAction::Retire {
                branch,
                expected,
                cancelled,
            } => self.retire(branch, &expected, cancelled),
            ModelAction::Close => self.close_result(),
            ModelAction::CapabilityLoss => self.close_result(),
        }
    }

    fn owner_is_open(&self) -> Result<(), ModelDenial> {
        (self.lifecycle == ModelOwnerLifecycle::Open)
            .then_some(())
            .ok_or(ModelDenial::OwnerUnavailable)
    }

    fn observe(&self, branch: BranchKey) -> ModelResult {
        if self.owner_is_open().is_err() {
            return ModelResult::Denied(ModelDenial::OwnerUnavailable);
        }
        match self.branch(branch) {
            None => ModelResult::Denied(ModelDenial::UnknownBranch),
            Some(branch) if branch.lifecycle == ModelBranchLifecycle::Retired => {
                ModelResult::Denied(ModelDenial::RetiredBranch)
            }
            Some(branch) => {
                ModelResult::Success(ModelSuccess::Observation(branch.observation.clone()))
            }
        }
    }

    fn readmit(&self, branch: BranchKey, expected: &ModelObservation) -> ModelResult {
        match self.observe(branch) {
            ModelResult::Success(observation) => match observation {
                ModelSuccess::Observation(current) if current == *expected => {
                    ModelResult::Success(ModelSuccess::Observation(current))
                }
                ModelSuccess::Observation(_) => ModelResult::Denied(ModelDenial::StaleBasis),
                _ => unreachable!("observe only returns observations"),
            },
            other => other,
        }
    }

    fn retain(&mut self, branch: BranchKey, observation: ModelObservation) -> ModelResult {
        if self.owner_is_open().is_err() {
            return ModelResult::Denied(ModelDenial::OwnerUnavailable);
        }
        match self.branch(branch) {
            None => ModelResult::Denied(ModelDenial::UnknownBranch),
            Some(branch) if branch.lifecycle == ModelBranchLifecycle::Retired => {
                ModelResult::Denied(ModelDenial::RetiredBranch)
            }
            Some(_) => {
                self.add_lease(branch, observation);
                ModelResult::Success(ModelSuccess::Lease)
            }
        }
    }

    fn readmit_retained(
        &self,
        branch: BranchKey,
        expected: &ModelObservation,
        lease: u64,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return ModelResult::Denied(ModelDenial::OwnerUnavailable);
        }
        let Some(held) = self.leases.get(&lease) else {
            return ModelResult::Denied(ModelDenial::MissingLease);
        };
        if held.branch != branch || held.observation != *expected {
            return ModelResult::Denied(ModelDenial::ForeignSnapshot);
        }
        ModelResult::Success(ModelSuccess::Observation(expected.clone()))
    }

    fn release(&mut self, lease: u64) -> ModelResult {
        if self.lifecycle != ModelOwnerLifecycle::Open {
            return ModelResult::Denied(ModelDenial::OwnerUnavailable);
        }
        self.leases
            .remove(&lease)
            .map(|_| ModelResult::Success(ModelSuccess::Release))
            .unwrap_or(ModelResult::Denied(ModelDenial::MissingLease))
    }

    fn fork(&mut self, source: BranchKey, child: BranchKey, child_name: String) -> ModelResult {
        if self.owner_is_open().is_err() {
            return ModelResult::Denied(ModelDenial::OwnerUnavailable);
        }
        let Some(source_branch) = self.live_branch(source).cloned() else {
            return match self.branch(source) {
                Some(_) => ModelResult::Denied(ModelDenial::RetiredBranch),
                None => ModelResult::Denied(ModelDenial::UnknownBranch),
            };
        };
        let mut observation = source_branch.observation;
        observation.branch = child;
        observation.restore_snapshot = None;
        observation.generation = 0;
        let branch = ModelBranch {
            key: child,
            parent: Some(source),
            name: child_name,
            observation: observation.clone(),
            lifecycle: ModelBranchLifecycle::Live,
        };
        self.branches.insert(child, branch);
        ModelResult::Success(ModelSuccess::Fork(observation))
    }

    fn close_result(&mut self) -> ModelResult {
        if self.lifecycle == ModelOwnerLifecycle::Open {
            self.close();
            ModelResult::Success(ModelSuccess::Closed)
        } else {
            ModelResult::Denied(ModelDenial::OwnerUnavailable)
        }
    }
}
