use super::{
    managed_views, WorthQueryPreImageRetentionWork, WorthQueryPrimaryGraphApplicationAttempt,
    WorthQueryPrimaryMutationWorkCounters,
};

/// All Query evidence whose owner effects crossed World's partial boundary.
/// The provider's exact unpublished entry owns this until a fresh World
/// successor performs or the recovery is explicitly retired.
pub(in crate::domain_computation::primary_graph::provider) struct ManagedUnpublishedAttempt {
    pub(super) publication_mode: ManagedUnpublishedPublicationMode,
    pub(super) attempt: WorthQueryPrimaryGraphApplicationAttempt,
    pub(super) work: WorthQueryPrimaryMutationWorkCounters,
    pub(super) index_maintenance_work: worth_relational::facade::indexes::DerivedIndexMaintenanceWork,
    pub(super) retained_preimage:
        Option<crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>,
    pub(super) preimage_retention_work: WorthQueryPreImageRetentionWork,
    pub(super) before: super::precommit_snapshot::WorthQueryPrecommitSnapshot,
    pub(super) managed_views: Option<managed_views::PreparedViewPublication>,
    pub(super) source_fact_rebase: Option<super::super::PreparedSourceFactRebase>,
    pub(super) source_fact_work_is_bounded: bool,
    pub(super) required_prerequisites:
        Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
    pub(super) prepared_lineage_slot:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,
    pub(super) prepared_output_witness:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness>,
    pub(super) prepared_touched_records: super::PreparedTouchedRecords,
    pub(super) lineage_metadata:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedLineageRecoveryMetadata>,
    pub(super) publication_admission:
        crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    pub(super) reserved_terminal:
        crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum ManagedUnpublishedPublicationMode {
    Ordinary,
    ConditionalDefinition,
}

impl ManagedUnpublishedAttempt {
    pub(in crate::domain_computation::primary_graph::provider) fn ordinary_adoption_supported(
        &self,
    ) -> bool {
        self.publication_mode == ManagedUnpublishedPublicationMode::Ordinary
    }
    pub(in crate::domain_computation::primary_graph::provider) fn publication_admission_mut(
        &mut self,
    ) -> &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission{
        &mut self.publication_admission
    }

    pub(in crate::domain_computation::primary_graph::provider) fn matches_fresh_binding(
        &self,
        idempotency: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        product: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.attempt.idempotency() == idempotency
            && self.attempt.affinity().operation_scope() == scope
            && self.attempt.affinity().product_publication().observation() == product
    }

    pub(in crate::domain_computation::primary_graph::provider) fn recovery_port(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldRecoveryPort {
        self.attempt.affinity().product_publication().recovery()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn prepare_fresh_terminal(
        &mut self,
        successor: worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
    ) -> Result<(), worth_relational::facade::mvcc::CompanionPreflightStop> {
        if self.reserved_terminal.matches_recovery_handle(&successor) {
            return Ok(());
        }
        let bytes = crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt::replacement_request_bytes()
            .ok_or(worth_relational::facade::mvcc::CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        self.publication_admission.admit_read_scratch(bytes)?;
        self.reserved_terminal.rebind_unfilled(successor);
        Ok(())
    }

    /// A World partial never authorizes its prospective Query settlement.
    /// Keep the handler's sealed facts while both old vacancies are queued for
    /// the owning metered admission to drain before a fresh reservation.
    pub(in crate::domain_computation::primary_graph::provider) fn cancel_stale_slot(&mut self) {
        if let Some(slot) = self.prepared_lineage_slot.take() {
            assert!(self.lineage_metadata.is_none());
            self.lineage_metadata = Some(slot.into_recovery_metadata());
        }
        if let Some(required) = self.required_prerequisites.as_mut() {
            required.cancel_reserved_identity_for_recovery();
        }
    }

    pub(in crate::domain_computation::primary_graph::provider) fn prepare_fresh_slot(
        &mut self,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        planned: &worth_runtime_world::facade::PlannedProductReferenceSuccessor,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial> {
        let Some(binding) = self.attempt.output_binding_type() else {
            assert!(self.required_prerequisites.is_none());
            return Ok(());
        };
        let scope = self.attempt.affinity().operation_scope();
        let partition = self.attempt.idempotency().source_partition_identity();
        if let Some(existing) = self.prepared_lineage_slot.as_ref() {
            if existing.matches_planned_successor(
                scope,
                binding,
                partition,
                planned,
                &mut self.publication_admission,
            )? {
                assert!(self.lineage_metadata.is_none());
                assert!(self.required_prerequisites.as_ref().is_none_or(|required| {
                    required.retains_reserved_identity(existing.identity())
                }));
                return Ok(());
            }
        }
        let mut slot =
            crate::domain_computation::primary_graph::output_lineage::prepare_output_lineage_slot(
                &provider.graph.output_lineage,
                scope,
                binding,
                partition,
                planned,
                &mut self.publication_admission,
            )?;
        if let Some(required) = self.required_prerequisites.as_mut() {
            required.replace_reserved_identity_for_recovery(
                slot.identity(),
                &mut self.publication_admission,
            )?;
        }
        if let Some(prior) = self.prepared_lineage_slot.take() {
            assert!(self.lineage_metadata.is_none());
            self.lineage_metadata = Some(prior.into_recovery_metadata());
        }
        if let Some(metadata) = self.lineage_metadata.take() {
            slot.retain_recovery_metadata(metadata);
        }
        self.prepared_lineage_slot = Some(slot);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::provider) fn complete(
        self,
        performed: worth_runtime_world::facade::PerformedCompositePublication,
        conditional_definition_generation: Option<u64>,
    ) -> super::WorthQueryCommittedApplicationSession {
        let product_publication = self
            .reserved_terminal
            .fill(performed.consume(), conditional_definition_generation);
        let next_basis = product_publication
            .publication()
            .commit()
            .basis()
            .relational_basis()
            .clone();
        let committed = product_publication
            .publication()
            .component_results()
            .retain_relational_commit_result()
            .expect("settled World adoption retains the original Relational result");
        super::WorthQueryCommittedApplicationSession {
            attempt: self.attempt,
            work: self.work,
            index_maintenance_work: self.index_maintenance_work,
            retained_preimage: self.retained_preimage,
            preimage_retention_work: self.preimage_retention_work,
            before: self.before.into_publication(),
            next_basis,
            committed,
            published_snapshot_custody: super::PublishedSnapshotCustody::SettledReleased,
            prepared_touched_records: Some(self.prepared_touched_records),
            product_publication,
            managed_views: self.managed_views,
            source_fact_rebase: self.source_fact_rebase,
            source_fact_admission: self
                .source_fact_work_is_bounded
                .then_some(self.publication_admission),
            required_prerequisites: self.required_prerequisites,
            prepared_lineage_slot: self.prepared_lineage_slot,
            prepared_output_witness: self.prepared_output_witness,
        }
    }
}
