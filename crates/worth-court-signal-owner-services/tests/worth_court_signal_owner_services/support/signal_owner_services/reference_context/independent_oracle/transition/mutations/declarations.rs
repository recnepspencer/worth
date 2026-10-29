pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::state::BranchKey, crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle, crate::signal_owner_services::reference_context::independent_oracle::state::ModelObservation, crate::signal_owner_services::reference_context::independent_oracle::state::ModelSnapshot, crate::signal_owner_services::reference_context::independent_oracle::state::ModelWorld};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult, crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess};

impl ModelWorld {
    pub(crate) fn advance(
        &mut self,
        branch: BranchKey,
        expected: &ModelObservation,
        cancelled: bool,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable);
        }
        if cancelled {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::Cancelled);
        }
        let Some(current) = self.branch(branch).cloned() else {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch);
        };
        if current.lifecycle == crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle::Retired {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch);
        }
        if current.observation != *expected {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis);
        }
        let mut next = current.observation;
        next.generation += 1;
        next.restore_snapshot = None;
        self.branch_mut(branch)
            .expect("the current branch remains present")
            .observation = next.clone();
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Advance(next))
    }

    pub(crate) fn capture(
        &mut self,
        branch: BranchKey,
        expected: &ModelObservation,
        snapshot: u64,
        cancelled: bool,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable);
        }
        if cancelled {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::Cancelled);
        }
        let Some(current) = self.branch(branch).cloned() else {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch);
        };
        if current.lifecycle == crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle::Retired {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch);
        }
        if current.observation != *expected {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis);
        }
        let mut next = current.observation;
        next.generation += 1;
        next.snapshot = Some(snapshot);
        next.restore_snapshot = None;
        let result_snapshot = ModelSnapshot { branch, snapshot };
        self.branch_mut(branch)
            .expect("the current branch remains present")
            .observation = next.clone();
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Capture {
            observation: next,
            snapshot: result_snapshot,
        })
    }

    pub(crate) fn restore(
        &mut self,
        branch: BranchKey,
        expected: &ModelObservation,
        snapshot: &ModelSnapshot,
        cancelled: bool,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable);
        }
        if cancelled {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::Cancelled);
        }
        let Some(current) = self.branch(branch).cloned() else {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch);
        };
        if current.lifecycle == crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle::Retired {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch);
        }
        if current.observation != *expected {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis);
        }
        if snapshot.branch != branch {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::ForeignSnapshot);
        }
        let mut next = current.observation;
        next.generation += 1;
        next.snapshot = Some(snapshot.snapshot);
        next.restore_snapshot = Some(snapshot.snapshot);
        self.branch_mut(branch)
            .expect("the current branch remains present")
            .observation = next.clone();
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Restore(next))
    }

    pub(crate) fn retire(
        &mut self,
        branch: BranchKey,
        expected: &ModelObservation,
        cancelled: bool,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable);
        }
        if cancelled {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::Cancelled);
        }
        let Some(current) = self.branch(branch).cloned() else {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch);
        };
        if branch == self.root {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::CurrentBranch);
        }
        if current.lifecycle == crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle::Retired {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch);
        }
        if current.observation != *expected {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis);
        }
        if self.has_retention_for(branch) {
            return crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetainedBasis);
        }
        self.branch_mut(branch)
            .expect("the current branch remains present")
            .lifecycle = crate::signal_owner_services::reference_context::independent_oracle::state::ModelBranchLifecycle::Retired;
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Retirement)
    }
}