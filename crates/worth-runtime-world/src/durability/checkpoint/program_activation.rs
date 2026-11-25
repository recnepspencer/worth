//! Read-only program facts from the same protected owner image and tails.
use super::{WorldCheckpointImage, WorldCheckpointReadSource};
use crate::durability::{
    WorldDurableHomeBinding, WorldDurableMediumBinding, WorldReadmissionDenial, WorldRecord,
};
use std::sync::Arc;
use worth_foundational::facade::DurableCompositeCommitIdentity;

#[derive(Debug)]
pub struct WorldProgramActivationHistory {
    pub(super) image: WorldCheckpointImage,
    _source: Box<dyn WorldCheckpointReadSource>,
    binding: Arc<WorldDurableMediumBinding>,
}
/// Borrows the original admitted record; it grants neither a writer nor a receipt.
pub struct WorldProgramActivationHistoryEntry<'a> {
    head: &'a super::image::WorldImageHead,
    binding: &'a WorldDurableHomeBinding,
}
impl WorldProgramActivationHistory {
    pub(super) fn capture(
        binding: Arc<WorldDurableMediumBinding>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldReadmissionDenial> {
        let source = binding
            .capture_checkpoint_source(request)
            .map_err(WorldReadmissionDenial::Medium)?;
        let (image, _) = WorldCheckpointImage::read_protected_source(
            source.as_ref(),
            binding.binding(),
            request,
            None,
        )?;
        Ok(Self {
            image,
            _source: source,
            binding,
        })
    }
    /// Whether durable history retirement removed this commit from live ancestry.
    pub fn is_retired(&self, commit: DurableCompositeCommitIdentity) -> bool {
        commit.owner() == self.binding.binding().owner()
            && self.image.retired_history.contains_key(&commit)
    }
    /// Whether durable branch retirement removed this exact branch incarnation.
    pub fn is_branch_retired(
        &self,
        incarnation: worth_foundational::facade::DurableProductBranchIncarnation,
    ) -> bool {
        self.image.retired_branches.contains_key(&incarnation)
    }
    /// Resolve `(incarnation, is_retired)` for a branch reference at `commit`,
    /// handling both direct commit creators and `reuse_exact` forks sharing `commit`
    /// through the indexed `(commit, incarnation)` map while charging lookup work.
    /// Prefers a live incarnation when the same branch name was later recreated.
    pub fn branch_status_for_commit(
        &self,
        commit: DurableCompositeCommitIdentity,
        branch: &str,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<(
            worth_foundational::facade::DurableProductBranchIncarnation,
            bool,
        )>,
        worth_execution::WorkCeilingDenial,
    > {
        request.consult()?;
        if commit.owner() != self.binding.binding().owner() {
            return Ok(None);
        }
        let mut retired_match = None;
        if let Some(head) = self.image.history.get(&commit) {
            let (head_branch, incarnation, _, _) = head.fields();
            if head_branch.name() == branch {
                if !self.image.retired_branches.contains_key(&incarnation) {
                    return Ok(Some((incarnation, false)));
                }
                retired_match = Some(incarnation);
            }
        }
        let mut cursor = (
            commit,
            worth_foundational::facade::DurableProductBranchIncarnation::from_recorded(
                commit.owner(),
                0,
            ),
        );
        while let Some(((found_commit, found_incarnation), head)) =
            self.image.commit_references.first_at_or_after(&cursor)
        {
            request.consult()?;
            if *found_commit != commit {
                break;
            }
            let (head_branch, incarnation, _, _) = head.fields();
            if head_branch.name() == branch {
                if !self.image.retired_branches.contains_key(&incarnation) {
                    return Ok(Some((incarnation, false)));
                }
                retired_match = Some(incarnation);
            }
            let Some(next_ordinal) = found_incarnation.ordinal().checked_add(1) else {
                break;
            };
            cursor = (
                commit,
                worth_foundational::facade::DurableProductBranchIncarnation::from_recorded(
                    commit.owner(),
                    next_ordinal,
                ),
            );
        }
        Ok(retired_match.map(|incarnation| (incarnation, true)))
    }
    /// Resolve `is_retired` for an exact `(commit, branch, lifecycle_ordinal)` reference
    /// through the indexed `(commit, incarnation)` map while charging lookup work.
    pub fn branch_status_for_commit_incarnation(
        &self,
        commit: DurableCompositeCommitIdentity,
        branch: &str,
        lifecycle_ordinal: u64,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<bool>, worth_execution::WorkCeilingDenial> {
        request.consult()?;
        if commit.owner() != self.binding.binding().owner() {
            return Ok(None);
        }
        let incarnation =
            worth_foundational::facade::DurableProductBranchIncarnation::from_recorded(
                commit.owner(),
                lifecycle_ordinal,
            );
        if let Some(head) = self.image.commit_references.get(&(commit, incarnation)) {
            let (head_branch, found_incarnation, _, _) = head.fields();
            if head_branch.name() == branch && found_incarnation == incarnation {
                return Ok(Some(self.image.retired_branches.contains_key(&incarnation)));
            }
        }
        Ok(None)
    }
    /// Descriptive history only: this neither mints a live commit nor admits use.
    pub fn recorded_entry(
        &self,
        commit: DurableCompositeCommitIdentity,
    ) -> Option<WorldProgramActivationHistoryEntry<'_>> {
        if commit.owner() != self.binding.binding().owner() {
            return None;
        }
        self.image
            .history
            .get(&commit)
            .map(|head| WorldProgramActivationHistoryEntry {
                head,
                binding: self.binding.binding(),
            })
    }
    pub fn entry(
        &self,
        commit: &crate::identity::CompositeCommitIdentity,
    ) -> Result<Option<WorldProgramActivationHistoryEntry<'_>>, WorldReadmissionDenial> {
        let owner = commit.owner_identity();
        if owner.durable() != Some(self.binding.binding().owner())
            || owner.home_incarnation_fence().is_none_or(|fence| {
                fence
                    .admit(self.binding.binding().incarnation().fence())
                    .is_err()
            })
        {
            return Err(WorldReadmissionDenial::ForeignOwner);
        }
        let key = DurableCompositeCommitIdentity::from_recorded(
            self.binding.binding().owner(),
            commit.ordinal(),
        );
        Ok(self
            .image
            .history
            .get(&key)
            .map(|head| WorldProgramActivationHistoryEntry {
                head,
                binding: self.binding.binding(),
            }))
    }
}
impl WorldProgramActivationHistoryEntry<'_> {
    pub fn branch(&self) -> &worth_foundational::facade::DurableProductBranchIdentity {
        self.head.fields().0
    }
    pub fn incarnation(&self) -> worth_foundational::facade::DurableProductBranchIncarnation {
        self.head.fields().1
    }
    pub fn generation(&self) -> worth_foundational::facade::DurableProductReferenceGeneration {
        self.head.fields().3
    }

    /// The original's actual execution attribution, unchanged by D16's target.
    pub fn recorded_revision(&self) -> [u8; 32] {
        match &self.head.program_record().record {
            WorldRecord::Branch {
                program_revision, ..
            } => *program_revision,
            WorldRecord::OpenAttempt(open) => open.program_revision,
            WorldRecord::BranchCreated(created) => created.intent.program_revision,
            WorldRecord::Owner { .. }
            | WorldRecord::Occurrence { .. }
            | WorldRecord::Released { .. }
            | WorldRecord::CreationIntent(_)
            | WorldRecord::CreationReleased { .. }
            | WorldRecord::Continued { .. }
            | WorldRecord::PlannedSignalLeg(_)
            | WorldRecord::BranchRetired { .. }
            | WorldRecord::HistoryRetired { .. } => {
                unreachable!("only admitted origins/opens own history entries")
            }
        }
    }
    /// Branch activation before this effect. Exact acknowledged canonical D16
    /// steps may run under A attribution after installing B; ordinary effects
    /// must match their recorded revision without an exemption.
    pub fn expected_activation(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], WorldReadmissionDenial> {
        match &self.head.program_record().record {
            WorldRecord::Branch {
                program_revision, ..
            } => Ok(*program_revision),
            WorldRecord::BranchCreated(created) => Ok(created.intent.program_revision),
            WorldRecord::OpenAttempt(open) => match &self.binding.lineage {
                Some(lineage) => lineage.recorded_activation_for_original(
                    &open.association,
                    open.program_revision,
                    request,
                ),
                None => Ok(open.program_revision),
            },
            WorldRecord::Owner { .. }
            | WorldRecord::Occurrence { .. }
            | WorldRecord::Released { .. }
            | WorldRecord::CreationIntent(_)
            | WorldRecord::CreationReleased { .. }
            | WorldRecord::Continued { .. }
            | WorldRecord::PlannedSignalLeg(_)
            | WorldRecord::BranchRetired { .. }
            | WorldRecord::HistoryRetired { .. } => {
                unreachable!("only admitted origins/opens own history entries")
            }
        }
    }
    pub fn predecessor(&self) -> Option<DurableCompositeCommitIdentity> {
        match &self.head.program_record().record {
            WorldRecord::Branch { .. } => None,
            WorldRecord::OpenAttempt(open) => Some(open.expected.commit),
            WorldRecord::BranchCreated(created) => Some(created.intent.source.commit),
            WorldRecord::Owner { .. }
            | WorldRecord::Occurrence { .. }
            | WorldRecord::Released { .. }
            | WorldRecord::CreationIntent(_)
            | WorldRecord::CreationReleased { .. }
            | WorldRecord::Continued { .. }
            | WorldRecord::PlannedSignalLeg(_)
            | WorldRecord::BranchRetired { .. }
            | WorldRecord::HistoryRetired { .. } => {
                unreachable!("only admitted origins/opens own history entries")
            }
        }
    }
}
