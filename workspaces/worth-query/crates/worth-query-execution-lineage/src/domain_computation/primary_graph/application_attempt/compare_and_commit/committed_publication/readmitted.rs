//! Readmitted World facts can describe an original, never recreate its delivery.
use super::*;
use crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxReadDenial as Denial;
use worth_runtime_world::facade::{
    CompositeCommitProvenance, CompositeRuntimeWorldCommit, WorldObservedHomePublication,
    WorldReadmittedPublication,
};

impl WorthQueryCommittedProductPublication {
    pub(crate) fn from_observed_home(
        original: &WorldObservedHomePublication,
        signal_publication: Option<CompositeSignalPublicationIdentity>,
    ) -> Result<Self, Denial> {
        if original.before().branch() != original.after().branch()
            || original.before().incarnation() != original.after().incarnation()
            || original.composite_commit().owner_identity().durable()
                != Some(original.after().branch().owner())
        {
            return Err(Denial::CommitMismatch);
        }
        match original.signal_posture() {
            CompositeComponentChangePosture::Published => {
                if original.signal_source().is_none() {
                    return Err(Denial::CommitMismatch);
                }
            }
            CompositeComponentChangePosture::RetainExact => {
                if signal_publication.is_some() || original.signal_source().is_some() {
                    return Err(Denial::CommitMismatch);
                }
            }
        }
        Ok(Self {
            custody: PublicationCustody::Described(Arc::new(DescriptiveProductPublication {
                product_branch: original.product_branch().clone(),
                product_incarnation: original.product_incarnation(),
                product_generation: original.product_generation(),
                composite_commit: original.composite_commit().clone(),
                publication_attempt: original.publication_attempt().clone(),
                relational_commit: original.native_receipt().clone(),
                relational_posture: CompositeComponentChangePosture::Published,
                signal_posture: original.signal_posture(),
                signal_publication,
                durable_signal_source: original.signal_source().cloned(),
                // A prior live conditional counter is not recovered source evidence.
                conditional_definition_generation: None,
            })),
        })
    }

    pub(crate) fn from_readmitted(
        original: &WorldReadmittedPublication,
        commit: &CompositeRuntimeWorldCommit,
    ) -> Result<Self, Denial> {
        let Some(native) = commit.relational_publication_identity() else {
            return Err(Denial::CommitMismatch);
        };
        if commit.identity() != original.after().selected_commit()
            || !matches!(commit.provenance(), CompositeCommitProvenance::Publication(attempt) if attempt == original.attempt_identity())
            || native.commit_id() != original.native_receipt().commit_id
            || native.version_id() != original.native_receipt().version_id
            || native.authoring_branch() != &original.native_receipt().branch_id
            || original.before().branch_identity() != original.after().branch_identity()
            || original.before().lifecycle_incarnation() != original.after().lifecycle_incarnation()
            || commit.relational_change() != CompositeComponentChangePosture::Published
        {
            return Err(Denial::CommitMismatch);
        }
        Ok(Self {
            custody: PublicationCustody::Described(Arc::new(DescriptiveProductPublication {
                product_branch: original.after().branch_identity().clone(),
                product_incarnation: original.after().lifecycle_incarnation(),
                product_generation: original.after().reference_generation(),
                composite_commit: commit.identity().clone(),
                publication_attempt: original.attempt_identity().clone(),
                relational_commit: original.native_receipt().clone(),
                relational_posture: commit.relational_change(),
                signal_posture: commit.signal_change(),
                signal_publication: commit.signal_publication_identity().cloned(),
                durable_signal_source: None,
                // A prior live conditional counter is not recovered source evidence.
                conditional_definition_generation: None,
            })),
        })
    }

    pub(crate) fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial> {
        match &mut self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .preflight_home_retirement(request)
                .map_err(Into::into),
            PublicationCustody::Described(_)
            | PublicationCustody::CoCommitted { .. }
            | PublicationCustody::SettledMovement { .. } => Ok(()),
        }
    }

    pub(crate) fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress,
        crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial,
    > {
        match &mut self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .advance_home_retirement(request, native)
                .map_err(Into::into),
            PublicationCustody::Described(_)
            | PublicationCustody::CoCommitted { .. }
            | PublicationCustody::SettledMovement { .. } => Ok(crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress::Complete),
        }
    }

    pub(crate) fn original_movement(
        &self,
    ) -> Option<&worth_runtime_world::facade::ConsumedCompositePublication> {
        self.performed().map(|receipt| receipt.publication())
    }

    pub(crate) fn follows_observation(&self, observation: &ProductBranchObservation) -> bool {
        match &self.custody {
            PublicationCustody::Performed(receipt) => {
                let before = receipt.publication().old_product_head();
                before.branch_identity() == observation.branch_identity()
                    && before.lifecycle_incarnation() == observation.lifecycle_incarnation()
                    && before.reference_generation() == observation.reference_generation()
                    && before.selected_commit() == observation.selected_commit()
                    && before.basis().relational_basis() == observation.basis().relational_basis()
            }
            PublicationCustody::SettledMovement { before, .. } => {
                before.matches_observation(observation)
            }
            PublicationCustody::Described(_) | PublicationCustody::CoCommitted { .. } => false,
        }
    }

    pub(crate) fn observed_product_identity(
        &self,
    ) -> Option<crate::basis::WorthQueryProductBranchReadIdentity> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => Some(
                crate::basis::WorthQueryProductBranchReadIdentity::from_reference_snapshot(
                    receipt.publication().new_product_head(),
                ),
            ),
            PublicationCustody::SettledMovement { after, .. } => Some(after.as_ref().clone()),
            PublicationCustody::Described(_) | PublicationCustody::CoCommitted { .. } => None,
        }
    }

    pub(crate) fn take_history(
        &mut self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryCommitHistoryHold,
    > {
        match &mut self.custody {
            PublicationCustody::Performed(receipt) => receipt.take_history(),
            PublicationCustody::Described(_)
            | PublicationCustody::CoCommitted { .. }
            | PublicationCustody::SettledMovement { .. } => None,
        }
    }

    /// Release performed World history and Native basis custody once a producer
    /// output settles into Ready cache, preserving exact descriptive endpoints.
    pub(crate) fn settle_for_ready_cache(&mut self) {
        let PublicationCustody::Performed(receipt) = &self.custody else {
            return;
        };
        let before = crate::basis::WorthQueryProductBranchReadIdentity::from_reference_snapshot(
            receipt.publication().old_product_head(),
        );
        let after = crate::basis::WorthQueryProductBranchReadIdentity::from_reference_snapshot(
            receipt.publication().new_product_head(),
        );
        let description = match self.descriptive().custody {
            PublicationCustody::Described(description) => description,
            _ => unreachable!("describing a performed publication yields immutable identities"),
        };
        self.custody = PublicationCustody::SettledMovement {
            description,
            before: Box::new(before),
            after: Box::new(after),
        };
    }

    pub(crate) fn original_movement_endpoints(
        &self,
    ) -> Option<(
        crate::basis::WorthQueryProductBranchReadIdentity,
        crate::basis::WorthQueryProductBranchReadIdentity,
        &CompositePublicationAttemptIdentity,
    )> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => Some((
                crate::basis::WorthQueryProductBranchReadIdentity::from_reference_snapshot(
                    receipt.publication().old_product_head(),
                ),
                crate::basis::WorthQueryProductBranchReadIdentity::from_reference_snapshot(
                    receipt.publication().new_product_head(),
                ),
                receipt.publication().attempt_identity(),
            )),
            PublicationCustody::SettledMovement {
                description,
                before,
                after,
            } => Some((
                before.as_ref().clone(),
                after.as_ref().clone(),
                &description.publication_attempt,
            )),
            PublicationCustody::Described(_) | PublicationCustody::CoCommitted { .. } => None,
        }
    }
}

/// Crate-boundary port for internal `WorthQueryCommittedProductPublication`
/// lifecycle, custody, and one-shot observation extraction. Never exported on
/// `worth-query-execution::facade` or Host.
pub trait CommittedProductPublicationInternalPort: Sized {
    fn from_observed_home(
        original: &WorldObservedHomePublication,
        signal_publication: Option<CompositeSignalPublicationIdentity>,
    ) -> Result<Self, Denial>;
    fn from_readmitted(
        original: &WorldReadmittedPublication,
        commit: &CompositeRuntimeWorldCommit,
    ) -> Result<Self, Denial>;
    fn from_receipt(receipt: WorthQueryProductPublicationReceipt) -> Self;
    fn co_committed_at(&self, observation: &ProductBranchObservation) -> bool;
    fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial>;
    fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress,
        crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial,
    >;
    fn original_movement(
        &self,
    ) -> Option<&worth_runtime_world::facade::ConsumedCompositePublication>;
    fn original_movement_endpoints(
        &self,
    ) -> Option<(
        crate::basis::WorthQueryProductBranchReadIdentity,
        crate::basis::WorthQueryProductBranchReadIdentity,
        &CompositePublicationAttemptIdentity,
    )>;
    fn settle_for_ready_cache(&mut self);
    fn follows_observation(&self, observation: &ProductBranchObservation) -> bool;
    fn observed_product_identity(
        &self,
    ) -> Option<crate::basis::WorthQueryProductBranchReadIdentity>;
    fn take_history(
        &mut self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryCommitHistoryHold,
    >;
    fn retired(&self) -> Self;
    fn descriptive(&self) -> Self;
    fn is_selected_at(&self, observation: &ProductBranchObservation) -> bool;
    fn retained_relational_basis(
        &self,
    ) -> Option<&worth_relational::facade::branch::AdmittedRelationalBranchBasis>;
    fn take_successor_observation(&self) -> Option<ProductBranchObservation>;
    fn take_output_demand_observation(&self) -> Option<ProductBranchObservation>;
    fn take_client_observation(&self) -> Option<ProductBranchObservation>;
}

impl CommittedProductPublicationInternalPort for WorthQueryCommittedProductPublication {
    fn from_observed_home(
        original: &WorldObservedHomePublication,
        signal_publication: Option<CompositeSignalPublicationIdentity>,
    ) -> Result<Self, Denial> {
        Self::from_observed_home(original, signal_publication)
    }

    fn from_readmitted(
        original: &WorldReadmittedPublication,
        commit: &CompositeRuntimeWorldCommit,
    ) -> Result<Self, Denial> {
        Self::from_readmitted(original, commit)
    }

    fn from_receipt(receipt: WorthQueryProductPublicationReceipt) -> Self {
        Self::from_receipt(receipt)
    }

    fn co_committed_at(&self, observation: &ProductBranchObservation) -> bool {
        Self::co_committed_at(self, observation)
    }

    fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial> {
        Self::preflight_home_retirement(self, request)
    }

    fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress,
        crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial,
    > {
        Self::advance_home_retirement(self, request, native)
    }

    fn original_movement(
        &self,
    ) -> Option<&worth_runtime_world::facade::ConsumedCompositePublication> {
        Self::original_movement(self)
    }

    fn original_movement_endpoints(
        &self,
    ) -> Option<(
        crate::basis::WorthQueryProductBranchReadIdentity,
        crate::basis::WorthQueryProductBranchReadIdentity,
        &CompositePublicationAttemptIdentity,
    )> {
        Self::original_movement_endpoints(self)
    }

    fn settle_for_ready_cache(&mut self) {
        Self::settle_for_ready_cache(self);
    }

    fn follows_observation(&self, observation: &ProductBranchObservation) -> bool {
        Self::follows_observation(self, observation)
    }

    fn observed_product_identity(
        &self,
    ) -> Option<crate::basis::WorthQueryProductBranchReadIdentity> {
        Self::observed_product_identity(self)
    }

    fn take_history(
        &mut self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryCommitHistoryHold,
    > {
        Self::take_history(self)
    }

    fn retired(&self) -> Self {
        Self::retired(self)
    }

    fn descriptive(&self) -> Self {
        Self::descriptive(self)
    }

    fn is_selected_at(&self, observation: &ProductBranchObservation) -> bool {
        Self::is_selected_at(self, observation)
    }

    fn retained_relational_basis(
        &self,
    ) -> Option<&worth_relational::facade::branch::AdmittedRelationalBranchBasis> {
        Self::retained_relational_basis(self)
    }

    fn take_successor_observation(&self) -> Option<ProductBranchObservation> {
        Self::take_successor_observation(self)
    }

    fn take_output_demand_observation(&self) -> Option<ProductBranchObservation> {
        Self::take_output_demand_observation(self)
    }

    fn take_client_observation(&self) -> Option<ProductBranchObservation> {
        Self::take_client_observation(self)
    }
}
