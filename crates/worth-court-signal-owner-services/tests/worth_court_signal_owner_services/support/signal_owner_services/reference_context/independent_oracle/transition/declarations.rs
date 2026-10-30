pub(crate) use {
    crate::signal_owner_services::reference_context::independent_oracle::state::BranchKey,
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranch,
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle,
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelObservation,
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle,
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelSnapshot,
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelWorld,
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

pub(crate) const STATE_CHANGING_OPERATIONS: [OperationKind; 9] = [
    crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Fork,
    crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Advance,
    crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Capture,
    crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Restore,
    crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Retain,
    crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Release,
    crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Retire,
    crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::Close,
    crate::signal_owner_services::reference_context::independent_oracle::transition::OperationKind::CapabilityLoss,
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
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Observe { branch } => self.observe(branch),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Readmit { branch, expected } => self.readmit(branch, &expected),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::ReadmitRetained {
                branch,
                expected,
                lease,
            } => self.readmit_retained(branch, &expected, lease),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retain {
                branch,
                observation,
            } => self.retain(branch, observation),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Release { lease } => self.release(lease),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Fork {
                source,
                child,
                child_name,
            } => self.fork(source, child, child_name),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Advance {
                branch,
                expected,
                cancelled,
            } => self.advance(branch, &expected, cancelled),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Capture {
                branch,
                expected,
                snapshot,
                cancelled,
            } => self.capture(branch, &expected, snapshot, cancelled),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Restore {
                branch,
                expected,
                snapshot,
                cancelled,
            } => self.restore(branch, &expected, &snapshot, cancelled),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Retire {
                branch,
                expected,
                cancelled,
            } => self.retire(branch, &expected, cancelled),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::Close => self.close_result(),
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelAction::CapabilityLoss => self.close_result(),
        }
    }

    pub(crate) fn owner_is_open(&self) -> Result<(), ModelDenial> {
        (self.lifecycle == crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle::Open)
            .then_some(())
            .ok_or(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable)
    }

    pub(crate) fn observe(&self, branch: BranchKey) -> ModelResult {
        if self.owner_is_open().is_err() {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable);
        }
        match self.branch(branch) {
            None => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch),
            Some(branch) if branch.lifecycle == crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle::Retired => {
                crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch)
            }
            Some(branch) => {
                crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Observation(branch.observation.clone()))
            }
        }
    }

    pub(crate) fn readmit(&self, branch: BranchKey, expected: &ModelObservation) -> ModelResult {
        match self.observe(branch) {
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(observation) => match observation {
                crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Observation(current) if current == *expected => {
                    crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Observation(current))
                }
                crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Observation(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis),
                _ => unreachable!("observe only returns observations"),
            },
            other => other,
        }
    }

    pub(crate) fn retain(
        &mut self,
        branch: BranchKey,
        observation: ModelObservation,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable);
        }
        match self.branch(branch) {
            None => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch),
            Some(branch) if branch.lifecycle == crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle::Retired => {
                crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch)
            }
            Some(_) => {
                self.add_lease(branch, observation);
                crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Lease)
            }
        }
    }

    pub(crate) fn readmit_retained(
        &self,
        branch: BranchKey,
        expected: &ModelObservation,
        lease: u64,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable);
        }
        let Some(held) = self.leases.get(&lease) else {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::MissingLease);
        };
        if held.branch != branch || held.observation != *expected {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::ForeignSnapshot);
        }
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Observation(expected.clone()))
    }

    pub(crate) fn release(&mut self, lease: u64) -> ModelResult {
        if self.lifecycle != crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle::Open {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable);
        }
        self.leases
            .remove(&lease)
            .map(|_| crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Release))
            .unwrap_or(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::MissingLease))
    }

    pub(crate) fn fork(
        &mut self,
        source: BranchKey,
        child: BranchKey,
        child_name: String,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable);
        }
        let Some(source_branch) = self.live_branch(source).cloned() else {
            return match self.branch(source) {
                Some(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch),
                None => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch),
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
            lifecycle: crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle::Live,
        };
        self.branches.insert(child, branch);
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Fork(observation))
    }

    pub(crate) fn close_result(&mut self) -> ModelResult {
        if self.lifecycle == crate::signal_owner_services::reference_context::independent_oracle::state::ModelOwnerLifecycle::Open {
            self.close();
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Closed)
        } else {
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable)
        }
    }
}
