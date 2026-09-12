use super::{CompactionReadInterlockDenial, CompactionRewritePublication};
use crate::{CurrentPhysicalRoot, PhysicalReadPlanCompletionReceipt};

/// Exact local plan completions on both sides of a published compaction.
///
/// This proves root and footprint correlation, not byte execution, live Store
/// protection, recovery visibility, or permission to reclaim physical media.
#[derive(Debug, Clone)]
pub struct CompactionReadPlanCompletion {
    publication: CompactionRewritePublication,
    pre_cutover: PhysicalReadPlanCompletionReceipt,
    post_cutover: PhysicalReadPlanCompletionReceipt,
}

impl CompactionReadPlanCompletion {
    const OWNER_CASE: super::CompactionOwnerCaseDeclaration =
        super::CompactionOwnerCaseDeclaration::declared_by_owner(
            super::CompactionOwnerCaseId::ValidateReadPlanCutover,
            super::CompactionCutoverState::PublicationCommitted,
            super::CompactionCutoverState::ReadPlanCutoverValidated,
        );

    pub fn from_publication(
        publication: CompactionRewritePublication,
        pre_cutover: PhysicalReadPlanCompletionReceipt,
        post_cutover: PhysicalReadPlanCompletionReceipt,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        let old_root = publication.publication().old_root();
        let new_root = publication.publication().new_root();
        if old_root.epoch() == new_root.epoch() {
            return Err(CompactionReadInterlockDenial::MixedRootDuringCompaction);
        }
        let plan = publication.delta().plan();
        let pre_release = pre_cutover.read_plan_release();
        if pre_release.root() != old_root
            || pre_release.footprint_basis() != plan.protected().footprint_basis()
        {
            return Err(CompactionReadInterlockDenial::PreCutoverReadReceiptMismatch);
        }
        let post_release = post_cutover.read_plan_release();
        if post_release.root() != new_root
            || post_release.root_epoch() != plan.target_epoch()
            || post_release.footprint_basis() != plan.candidates().footprint_basis()
        {
            return Err(CompactionReadInterlockDenial::PostCutoverReadReceiptMismatch);
        }
        Ok(Self {
            publication,
            pre_cutover,
            post_cutover,
        })
    }

    pub const fn cutover_state(&self) -> super::CompactionCutoverState {
        super::CompactionCutoverState::ReadPlanCutoverValidated
    }

    pub const fn owner_case_observation(&self) -> super::CompactionOwnerCaseObservation {
        super::CompactionOwnerCaseObservation::issued_by_owner(Self::OWNER_CASE)
    }

    pub const fn publication(&self) -> &CompactionRewritePublication {
        &self.publication
    }

    pub const fn pre_cutover_root(&self) -> CurrentPhysicalRoot {
        self.publication.publication().old_root()
    }

    pub const fn post_cutover_root(&self) -> CurrentPhysicalRoot {
        self.publication.publication().new_root()
    }

    pub const fn old_reachability_deferred(&self) -> bool {
        self.publication.delta().plan().reclaim_deferred()
    }

    pub const fn pre_cutover_plan_completion(&self) -> PhysicalReadPlanCompletionReceipt {
        self.pre_cutover
    }

    pub const fn post_cutover_plan_completion(&self) -> PhysicalReadPlanCompletionReceipt {
        self.post_cutover
    }
}

pub(super) fn owner_cases() -> impl Iterator<Item = super::CompactionOwnerCaseDeclaration> {
    std::iter::once(CompactionReadPlanCompletion::OWNER_CASE)
}
