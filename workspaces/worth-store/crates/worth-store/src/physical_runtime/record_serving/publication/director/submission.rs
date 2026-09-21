use std::sync::Weak;

use worth_proof::NonEmpty;
use worth_proof::TransitionOutcome;

use super::RecordPublicationDirector;
use crate::physical_runtime::{
    record_serving::{
        publication::{
            PhysicalManifestCapacityTransition, PhysicalMutationPreparationOutcome,
            PhysicalMutationPreparationStale,
        },
        AdmittedRecordPlacementPolicy, RecordAppendBatch,
    },
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationIdempotencyKey,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationRequest,
};

#[derive(Clone)]
pub struct PhysicalRecordSubmission {
    director: Weak<RecordPublicationDirector>,
}

#[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
impl PhysicalRecordSubmission {
    pub(super) const fn new(director: Weak<RecordPublicationDirector>) -> Self {
        Self { director }
    }

    pub fn issue_idempotency_key(
        &self,
        material: PhysicalMutationIdempotencyMaterial,
    ) -> Result<PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyIssuanceDenial> {
        let director = self
            .director
            .upgrade()
            .ok_or(PhysicalMutationIdempotencyIssuanceDenial::DurabilityAuthorityReleased)?;
        director.idempotency.issue_key(material)
    }

    pub fn prepare_durable_append(
        &self,
        batch: RecordAppendBatch,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let director = match self.director.upgrade() {
            Some(director) => director,
            None => {
                return TransitionOutcome::stale(
                    PhysicalMutationPreparationStale::PublicationAuthorityReleased,
                )
                .into()
            }
        };
        director.prepare_durable_append(
            batch,
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            request,
        )
    }

    pub fn prepare_durable_append_with_manifest_capacity_transition(
        &self,
        batch: RecordAppendBatch,
        placement: AdmittedRecordPlacementPolicy,
        manifest_capacity_transition: PhysicalManifestCapacityTransition,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let director = match self.director.upgrade() {
            Some(director) => director,
            None => {
                return TransitionOutcome::stale(
                    PhysicalMutationPreparationStale::PublicationAuthorityReleased,
                )
                .into()
            }
        };
        director.prepare_durable_append(batch, placement, manifest_capacity_transition, request)
    }

    pub fn rewrite_selected_inline_segment(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let director = match self.director.upgrade() {
            Some(director) => director,
            None => {
                return TransitionOutcome::stale(
                    PhysicalMutationPreparationStale::PublicationAuthorityReleased,
                )
                .into()
            }
        };
        director.prepare_selected_segment_rewrite(placement, request)
    }

    pub(in crate::physical_runtime) fn cancel_prepared_before_group_seal(
        &self,
        prepared: crate::physical_runtime::PreparedPhysicalMutation,
    ) -> crate::physical_runtime::PhysicalPreSealCancellationOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalPreSealCancellationOutcome::NotCancelled {
                prepared,
                cause: crate::physical_runtime::PhysicalPreSealCancellationDenial::
                    DurabilityAuthorityReleased,
            };
        };
        director.cancel_prepared_before_group_seal(prepared)
    }

    pub(in crate::physical_runtime) fn append_prepared_wal_group(
        &self,
        members: NonEmpty<crate::physical_runtime::PreparedPhysicalMutation>,
    ) -> crate::physical_runtime::PhysicalWalGroupAppendOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalWalGroupAppendOutcome::NotAdmitted {
                members,
                cause: crate::physical_runtime::PhysicalWalGroupAppendFailureCause::RuntimeReleased,
            };
        };
        let members = match director.plan_prepared_group_for_wal(members) {
            Ok(members) => members,
            Err((members, denial)) => {
                return crate::physical_runtime::PhysicalWalGroupAppendOutcome::NotAdmitted {
                    members,
                    cause: crate::physical_runtime::PhysicalWalGroupAppendFailureCause::Reservation(
                        crate::physical_runtime::PhysicalWalReservationDenial::DataPlanning(denial),
                    ),
                }
            }
        };
        director.wal.append_prepared_group(members)
    }

    pub(in crate::physical_runtime) fn continue_prepared_wal_group(
        &self,
        continuation: crate::physical_runtime::PhysicalWalGroupAppendContinuation,
    ) -> crate::physical_runtime::PhysicalWalGroupAppendOutcome {
        let Some(director) = self.director.upgrade() else {
            return continuation.runtime_released();
        };
        director.wal.continue_prepared_group(continuation)
    }

    pub fn wal_observation(&self) -> Option<crate::physical_runtime::PhysicalWalObservation> {
        self.director
            .upgrade()
            .map(|director| director.wal.observation())
    }

    pub(in crate::physical_runtime) fn synchronize_appended_wal_group(
        &self,
        appended: crate::physical_runtime::SealedPhysicalDurabilityGroupMembers,
    ) -> crate::physical_runtime::PhysicalWalGroupBarrierOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalWalGroupBarrierOutcome::BarrierNotStarted {
                appended,
                cause:
                    crate::physical_runtime::PhysicalWalGroupBarrierFailureCause::RuntimeReleased,
            };
        };
        director.wal_barrier.synchronize_appended_group(appended)
    }

    pub(in crate::physical_runtime) fn dispatch_wal_durable_data(
        &self,
        durable: crate::physical_runtime::WalDurablePhysicalMutation,
    ) -> crate::physical_runtime::PhysicalDataDispatchOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalDataDispatchOutcome::NotStarted {
                durable,
                cause:
                    crate::physical_runtime::PhysicalDataDispatchFailureCause::PublicationAuthorityReleased,
            };
        };
        director.dispatch_wal_durable_data(durable)
    }

    pub(in crate::physical_runtime) fn join_data_settled_group(
        &self,
        basis: crate::physical_runtime::PhysicalDurabilityGroupBasis,
        members: NonEmpty<crate::physical_runtime::DataSettledPhysicalMutation>,
    ) -> crate::physical_runtime::PhysicalDataSettledGroupAdmissionOutcome {
        let Some(director) = self.director.upgrade() else {
            return Err(
                crate::physical_runtime::RejectedDataSettledPhysicalMutationMembers::
                    runtime_released(members),
            );
        };
        let identity = members.as_slice()[0].mutation_identity();
        if identity.store_identity() != director.durability.store_identity() {
            return Err(
                crate::physical_runtime::RejectedDataSettledPhysicalMutationMembers::foreign_store(
                    members,
                ),
            );
        }
        if identity.runtime_identity() != director.durability.runtime_identity() {
            return Err(
                crate::physical_runtime::RejectedDataSettledPhysicalMutationMembers::stale_runtime(
                    members,
                ),
            );
        }
        crate::physical_runtime::DataSettledPhysicalMutationMembers::admit(basis, members)
    }

    pub(in crate::physical_runtime) fn prepare_root_publication(
        &self,
        settled: crate::physical_runtime::DataSettledPhysicalMutationMembers,
    ) -> crate::physical_runtime::PhysicalRootPublicationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalRootPublicationPreparationOutcome::
                runtime_released(settled);
        };
        crate::physical_runtime::PhysicalRootPublicationPreparationOutcome::from_result(
            director.prepare_settled_root_publication(settled),
        )
    }

    pub(in crate::physical_runtime) fn continue_root_publication_preparation(
        &self,
        planning: crate::physical_runtime::RootPublicationPlanningMembers,
    ) -> crate::physical_runtime::PhysicalRootPublicationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalRootPublicationPreparationOutcome::from_result(
                Err(
                    crate::physical_runtime::durability::PhysicalRootPublicationPreparationFailure::
                        PlanningAuthorityReleased { planning },
                ),
            );
        };
        crate::physical_runtime::PhysicalRootPublicationPreparationOutcome::from_result(
            director.continue_root_publication_preparation(planning),
        )
    }

    pub(in crate::physical_runtime) fn continue_root_publication_candidate(
        &self,
        candidate: crate::physical_runtime::RootPublicationCandidatePlan,
    ) -> crate::physical_runtime::PhysicalRootPublicationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalRootPublicationPreparationOutcome::from_result(
                Err(
                    crate::physical_runtime::durability::PhysicalRootPublicationPreparationFailure::
                        CandidateAuthorityReleased { candidate },
                ),
            );
        };
        crate::physical_runtime::PhysicalRootPublicationPreparationOutcome::from_result(
            director.continue_root_publication_candidate(candidate),
        )
    }

    pub(in crate::physical_runtime) fn replace_prepared_root(
        &self,
        prepared: crate::physical_runtime::RootPublicationPreparedPhysicalMutationMembers,
    ) -> crate::physical_runtime::PhysicalRootReplacementOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalRootReplacementOutcome::runtime_released(
                prepared,
            );
        };
        director.replace_prepared_root(prepared)
    }

    pub(in crate::physical_runtime) fn synchronize_replaced_root_namespace(
        &self,
        replaced: crate::physical_runtime::RootReplacedPhysicalMutationMembers,
    ) -> crate::physical_runtime::PhysicalRootNamespaceDurabilityOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalRootNamespaceDurabilityOutcome::
                runtime_released(replaced);
        };
        director.synchronize_replaced_root_namespace(replaced)
    }

    pub(in crate::physical_runtime) fn advance_namespace_durable_root(
        &self,
        durable: crate::physical_runtime::RootNamespaceDurablePhysicalMutationMembers,
    ) -> crate::physical_runtime::PhysicalCurrentRootAdvanceOutcome {
        let Some(director) = self.director.upgrade() else {
            return crate::physical_runtime::PhysicalCurrentRootAdvanceOutcome::
                InspectionRequired(
                    crate::physical_runtime::IndeterminatePhysicalCurrentRootAdvance::
                        publication_authority_released(durable),
                );
        };
        director.advance_namespace_durable_root(durable)
    }
}
