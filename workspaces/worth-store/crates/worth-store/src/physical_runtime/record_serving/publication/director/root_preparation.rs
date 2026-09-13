use worth_store_physical_format::RecordArtifactFile;

use super::RecordPublicationDirector;
use crate::physical_runtime::durability::{
    PhysicalRootPublicationPreparationFailure as RootPublicationPreparationFailure,
    PhysicalRootPublicationPreparationNotStartedCause as RootPublicationPreparationNotStartedCause,
};
use crate::physical_runtime::record_serving::{
    planning::rebased_root::{project_settled_root, RootRebaseContext},
    publication::append::next_nonzero_random,
    RecordAppendDenial, RecordAppendError,
};
use crate::physical_runtime::{
    DataSettledPhysicalMutationMembers, RootPublicationCandidatePlan,
    RootPublicationPlanningMembers, RootPublicationPreparedPhysicalMutationMembers,
};

impl RecordPublicationDirector {
    pub(super) fn prepare_settled_root_publication(
        &self,
        settled: DataSettledPhysicalMutationMembers,
    ) -> Result<RootPublicationPreparedPhysicalMutationMembers, RootPublicationPreparationFailure>
    {
        let Some(runtime) = self.runtime.upgrade() else {
            return Err(RootPublicationPreparationFailure::NotStarted {
                settled,
                cause: RootPublicationPreparationNotStartedCause::RuntimeReleased,
            });
        };
        let (current_root, current_free_space) = self.root_owner.snapshot();
        if settled
            .members()
            .iter()
            .any(|member| member.prepared_root_source_generation() != current_root.generation())
        {
            return Err(RootPublicationPreparationFailure::NotStarted {
                settled,
                cause: RootPublicationPreparationNotStartedCause::CurrentRootMismatch,
            });
        }
        let planning = RootPublicationPlanningMembers::from_settled(settled);
        self.prepare_root_publication_planning(runtime, current_root, current_free_space, planning)
    }

    #[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
    pub(super) fn continue_root_publication_preparation(
        &self,
        planning: RootPublicationPlanningMembers,
    ) -> Result<RootPublicationPreparedPhysicalMutationMembers, RootPublicationPreparationFailure>
    {
        let Some(runtime) = self.runtime.upgrade() else {
            return Err(RootPublicationPreparationFailure::PlanningAuthorityReleased { planning });
        };
        let (current_root, current_free_space) = self.root_owner.snapshot();
        if planning.source_root_generation() != current_root.generation() {
            return Err(RootPublicationPreparationFailure::TransitionDenied {
                planning,
                cause: crate::physical_runtime::PhysicalRootPublicationTransitionDenial::
                    CurrentRootMismatch,
            });
        }
        self.prepare_root_publication_planning(runtime, current_root, current_free_space, planning)
    }

    fn prepare_root_publication_planning(
        &self,
        runtime: std::sync::Arc<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
        current_root: worth_store_physical_format::DurablePhysicalRootManifest,
        current_free_space: worth_store_physical_format::DurableFreeSpaceManifestHeader,
        planning: RootPublicationPlanningMembers,
    ) -> Result<RootPublicationPreparedPhysicalMutationMembers, RootPublicationPreparationFailure>
    {
        let planning = match planning.merge_candidate_projections() {
            Ok(planning) => planning,
            Err((planning, rejected)) => {
                return Err(RootPublicationPreparationFailure::ProjectionRejected {
                    planning,
                    rejected,
                })
            }
        };
        let group = planning.group_basis();
        let candidate_publication = match next_nonzero_random() {
            Ok(candidate) => candidate,
            Err(cause) => {
                return Err(RootPublicationPreparationFailure::Planning { planning, cause })
            }
        };
        let identity = match crate::physical_runtime::durability::PhysicalRootPublicationIdentity::
            from_settled_group(
                self.durability.store_identity(),
                self.durability.runtime_identity(),
                self.durability.policy_identity(),
                group,
                current_root.generation(),
                candidate_publication,
            ) {
            Some(identity) => identity,
            None => {
                return Err(RootPublicationPreparationFailure::Planning {
                    planning,
                    cause: RecordAppendError::Denied(RecordAppendDenial::RootGenerationExhausted),
                })
            }
        };
        let transition = match self.root_owner.begin(identity, current_root.clone()) {
            Ok(transition) => transition,
            Err(cause) => {
                return Err(RootPublicationPreparationFailure::TransitionDenied { planning, cause })
            }
        };
        let allocation_bytes = planning.allocation_bytes();
        let allocation = match self
            .residency
            .begin_foreground_write_operation(allocation_bytes)
        {
            Ok(allocation) => allocation,
            Err(denial) => {
                return Err(RootPublicationPreparationFailure::Planning {
                    planning,
                    cause: RecordAppendError::Denied(RecordAppendDenial::from_residency(denial)),
                })
            }
        };
        let candidate = RecordArtifactFile::CatalogCandidate {
            publication: candidate_publication,
        };
        let (planning_core, prepared, allocation_bytes) = planning.into_merged_parts();
        let placement = prepared.placement;
        let capacity_transition = prepared.manifest_capacity_transition;
        let (plan, successor_free_space) = match project_settled_root(
            prepared,
            RootRebaseContext {
                allocation: &allocation,
                media: runtime.executor.record_serving_media(),
                residency: self.residency.clone(),
                format: self.format,
                access: self.access,
                current_root: &current_root,
                current_free_space: &current_free_space,
                placement,
                capacity_transition,
            },
            candidate,
        ) {
            Ok(plan) => plan,
            Err((prepared, cause)) => {
                return Err(RootPublicationPreparationFailure::Planning {
                    planning: RootPublicationPlanningMembers::from_merged_parts(
                        planning_core,
                        prepared,
                        allocation_bytes,
                    ),
                    cause,
                })
            }
        };
        let candidate = RootPublicationCandidatePlan::new(
            planning_core,
            current_root,
            successor_free_space,
            allocation_bytes,
            transition,
            plan,
        );
        self.prepare_root_publication_candidate(runtime, candidate, allocation)
    }
}
