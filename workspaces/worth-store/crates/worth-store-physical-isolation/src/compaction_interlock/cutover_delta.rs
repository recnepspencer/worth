use super::{CompactionReadInterlockDenial, CompactionReadInterlockPlan};
use crate::{CurrentPhysicalRoot, PhysicalPublicationPlanCompletion};

#[derive(Debug, Clone)]
pub struct CompactionCutoverDelta {
    plan: CompactionReadInterlockPlan,
    rewritten_root: CurrentPhysicalRoot,
}

impl CompactionCutoverDelta {
    const OWNER_CASE: super::CompactionOwnerCaseDeclaration =
        super::CompactionOwnerCaseDeclaration::declared_by_owner(
            super::CompactionOwnerCaseId::LowerRewrite,
            super::CompactionCutoverState::PlanAdmitted,
            super::CompactionCutoverState::RewriteLowered,
        );

    pub const fn cutover_state(&self) -> super::CompactionCutoverState {
        super::CompactionCutoverState::RewriteLowered
    }

    pub const fn owner_case_observation(&self) -> super::CompactionOwnerCaseObservation {
        super::CompactionOwnerCaseObservation::issued_by_owner(Self::OWNER_CASE)
    }

    pub fn lower(
        plan: CompactionReadInterlockPlan,
        rewritten_root: CurrentPhysicalRoot,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        if rewritten_root.epoch() != plan.target_epoch() {
            return Err(CompactionReadInterlockDenial::StaleEpochReuse {
                source_epoch: plan.source_epoch(),
                reused_epoch: rewritten_root.epoch(),
            });
        }
        Ok(Self {
            plan,
            rewritten_root,
        })
    }

    pub fn lower_to_manifest(
        plan: CompactionReadInterlockPlan,
        target_manifest_epoch: u64,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        let source_manifest = plan.protected().root().manifest_epoch();
        let target_manifest =
            crate::ManifestEpoch::from_admitted_physical_basis(target_manifest_epoch);
        if target_manifest_epoch <= source_manifest.get() {
            return Err(CompactionReadInterlockDenial::StaleManifestEpochReuse {
                source_epoch: source_manifest,
                reused_epoch: target_manifest,
            });
        }
        let rewritten_root = crate::CurrentPhysicalRoot::from_physical_isolation_entry(
            crate::CurrentPhysicalRootBasis::new(
                plan.target_epoch(),
                target_manifest,
                plan.protected().root().store_authority_identity(),
            ),
            crate::PhysicalOrderingContract::root_swap_acquire_release(),
        )
        .map_err(|_| CompactionReadInterlockDenial::SourceEvidenceMismatch)?;
        Self::lower(plan, rewritten_root)
    }

    pub(crate) fn bind_publication(
        self,
        publication: &PhysicalPublicationPlanCompletion,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        if publication.old_root() != self.plan.protected().root()
            || publication.new_root() != self.rewritten_root
        {
            return Err(CompactionReadInterlockDenial::PublicationRootMismatch);
        }
        if !publication.old_reachability().retained_until_release() {
            return Err(CompactionReadInterlockDenial::MissingOldRootPreservation);
        }
        if publication.old_reachability().footprint_basis()
            != self.plan.protected().footprint_basis()
        {
            return Err(
                CompactionReadInterlockDenial::PublicationReachabilityFootprintMismatch {
                    protected: self.plan.protected().footprint_basis(),
                    preserved: publication.old_reachability().footprint_basis(),
                },
            );
        }
        Ok(self)
    }

    pub const fn plan(&self) -> &CompactionReadInterlockPlan {
        &self.plan
    }

    pub const fn rewritten_root(&self) -> CurrentPhysicalRoot {
        self.rewritten_root
    }
}

pub(super) fn owner_cases() -> impl Iterator<Item = super::CompactionOwnerCaseDeclaration> {
    std::iter::once(CompactionCutoverDelta::OWNER_CASE)
}
