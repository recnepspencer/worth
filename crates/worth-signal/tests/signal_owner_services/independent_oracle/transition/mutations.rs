use super::super::state::{
    BranchKey, ModelBranchLifecycle, ModelObservation, ModelSnapshot, ModelWorld,
};
use super::{ModelDenial, ModelResult, ModelSuccess};

impl ModelWorld {
    pub(super) fn advance(
        &mut self,
        branch: BranchKey,
        expected: &ModelObservation,
        cancelled: bool,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return ModelResult::Denied(ModelDenial::OwnerUnavailable);
        }
        if cancelled {
            return ModelResult::Denied(ModelDenial::Cancelled);
        }
        let Some(current) = self.branch(branch).cloned() else {
            return ModelResult::Denied(ModelDenial::UnknownBranch);
        };
        if current.lifecycle == ModelBranchLifecycle::Retired {
            return ModelResult::Denied(ModelDenial::RetiredBranch);
        }
        if current.observation != *expected {
            return ModelResult::Denied(ModelDenial::StaleBasis);
        }
        let mut next = current.observation;
        next.generation += 1;
        next.restore_snapshot = None;
        self.branch_mut(branch)
            .expect("the current branch remains present")
            .observation = next.clone();
        ModelResult::Success(ModelSuccess::Advance(next))
    }

    pub(super) fn capture(
        &mut self,
        branch: BranchKey,
        expected: &ModelObservation,
        snapshot: u64,
        cancelled: bool,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return ModelResult::Denied(ModelDenial::OwnerUnavailable);
        }
        if cancelled {
            return ModelResult::Denied(ModelDenial::Cancelled);
        }
        let Some(current) = self.branch(branch).cloned() else {
            return ModelResult::Denied(ModelDenial::UnknownBranch);
        };
        if current.lifecycle == ModelBranchLifecycle::Retired {
            return ModelResult::Denied(ModelDenial::RetiredBranch);
        }
        if current.observation != *expected {
            return ModelResult::Denied(ModelDenial::StaleBasis);
        }
        let mut next = current.observation;
        next.generation += 1;
        next.snapshot = Some(snapshot);
        next.restore_snapshot = None;
        let result_snapshot = ModelSnapshot { branch, snapshot };
        self.branch_mut(branch)
            .expect("the current branch remains present")
            .observation = next.clone();
        ModelResult::Success(ModelSuccess::Capture {
            observation: next,
            snapshot: result_snapshot,
        })
    }

    pub(super) fn restore(
        &mut self,
        branch: BranchKey,
        expected: &ModelObservation,
        snapshot: &ModelSnapshot,
        cancelled: bool,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return ModelResult::Denied(ModelDenial::OwnerUnavailable);
        }
        if cancelled {
            return ModelResult::Denied(ModelDenial::Cancelled);
        }
        let Some(current) = self.branch(branch).cloned() else {
            return ModelResult::Denied(ModelDenial::UnknownBranch);
        };
        if current.lifecycle == ModelBranchLifecycle::Retired {
            return ModelResult::Denied(ModelDenial::RetiredBranch);
        }
        if current.observation != *expected {
            return ModelResult::Denied(ModelDenial::StaleBasis);
        }
        if snapshot.branch != branch {
            return ModelResult::Denied(ModelDenial::ForeignSnapshot);
        }
        let mut next = current.observation;
        next.generation += 1;
        next.snapshot = Some(snapshot.snapshot);
        next.restore_snapshot = Some(snapshot.snapshot);
        self.branch_mut(branch)
            .expect("the current branch remains present")
            .observation = next.clone();
        ModelResult::Success(ModelSuccess::Restore(next))
    }

    pub(super) fn retire(
        &mut self,
        branch: BranchKey,
        expected: &ModelObservation,
        cancelled: bool,
    ) -> ModelResult {
        if self.owner_is_open().is_err() {
            return ModelResult::Denied(ModelDenial::OwnerUnavailable);
        }
        if cancelled {
            return ModelResult::Denied(ModelDenial::Cancelled);
        }
        let Some(current) = self.branch(branch).cloned() else {
            return ModelResult::Denied(ModelDenial::UnknownBranch);
        };
        if branch == self.root {
            return ModelResult::Denied(ModelDenial::CurrentBranch);
        }
        if current.lifecycle == ModelBranchLifecycle::Retired {
            return ModelResult::Denied(ModelDenial::RetiredBranch);
        }
        if current.observation != *expected {
            return ModelResult::Denied(ModelDenial::StaleBasis);
        }
        if self.has_retention_for(branch) {
            return ModelResult::Denied(ModelDenial::RetainedBasis);
        }
        self.branch_mut(branch)
            .expect("the current branch remains present")
            .lifecycle = ModelBranchLifecycle::Retired;
        ModelResult::Success(ModelSuccess::Retirement)
    }
}
