//! Borrowed owner access across the sealed Relational package boundary.
use super::*;
use worth_relational_contracts::internal_port as contracts;

#[doc(hidden)]
pub trait PreparedRelationalCommitCandidateOwnerPort: Sized {
    fn reservation_count(&self) -> usize;
    fn change_summary_basis_and_roots(
        &self,
    ) -> Option<(
        RelationalBranchBasisDescriptor,
        Arc<RelationalBranchRoot>,
        Arc<RelationalBranchRoot>,
    )>;

    fn runtime_instance_id(&self) -> u64;

    fn belongs_to_receiver_epoch(
        &self,
        epoch: &worth_relational_mutation::internal_port::runtime::state::runtime_state::configuration_snapshot::RelationalRuntimeConfigurationSnapshot,
    ) -> bool;

    fn configuration_invariant_generation(&self) -> u64;

    fn expected_basis(&self) -> Option<RelationalBranchBasisDescriptor>;

    fn with_successor_shape<R>(
        &self,
        read: impl FnOnce(
            &RelationalBranchBasisDescriptor,
            usize,
            u64,
            contracts::history::data::coordinates::CommitId,
        ) -> R,
    ) -> Option<R>;

    fn belongs_to_publication_owner(
        &self,
        binding: &crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding,
    ) -> bool;

    fn index_preparation_roots(
        &self,
    ) -> Option<(Arc<RelationalBranchRoot>, Arc<RelationalBranchRoot>)>;

    fn recovered_codec_publication_custody(&self) -> Option<Arc<RecoveredCodecPublicationCustody>>;

    fn home_step(&self) -> Option<Arc<AdmittedRelationalHomeProgramStep>>;

    fn home_index_definitions(&self) -> Option<Arc<PreparedRelationalIndexDefinitions>>;

    fn covers_home_definitions(
        &self,
        definitions: &PreparedRelationalIndexDefinitions,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, worth_execution::WorkCeilingDenial>;

    fn indexes_already_prepared(&self) -> bool;

    fn attach_prepared_indexes(
        &mut self,
        prepared: worth_relational_indexes::internal_port::indexes::authority::maintenance::candidate_publication::PreparedCandidateIndexPublication,
    ) -> Result<(), contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind>;
}

impl PreparedRelationalCommitCandidateOwnerPort for PreparedRelationalCommitCandidate {
    fn reservation_count(&self) -> usize {
        self.reservation_count()
    }
    fn change_summary_basis_and_roots(
        &self,
    ) -> Option<(
        RelationalBranchBasisDescriptor,
        Arc<RelationalBranchRoot>,
        Arc<RelationalBranchRoot>,
    )> {
        self.change_summary_basis_and_roots()
    }

    fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id()
    }

    fn belongs_to_receiver_epoch(
        &self,
        epoch: &worth_relational_mutation::internal_port::runtime::state::runtime_state::configuration_snapshot::RelationalRuntimeConfigurationSnapshot,
    ) -> bool {
        self.belongs_to_receiver_epoch(epoch)
    }

    fn configuration_invariant_generation(&self) -> u64 {
        self.configuration_invariant_generation()
    }

    fn expected_basis(&self) -> Option<RelationalBranchBasisDescriptor> {
        self.expected_basis()
    }

    fn with_successor_shape<R>(
        &self,
        read: impl FnOnce(
            &RelationalBranchBasisDescriptor,
            usize,
            u64,
            contracts::history::data::coordinates::CommitId,
        ) -> R,
    ) -> Option<R> {
        self.with_successor_shape(read)
    }

    fn belongs_to_publication_owner(
        &self,
        binding: &crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding,
    ) -> bool {
        self.belongs_to_publication_owner(binding)
    }

    fn index_preparation_roots(
        &self,
    ) -> Option<(Arc<RelationalBranchRoot>, Arc<RelationalBranchRoot>)> {
        self.index_preparation_roots()
    }

    fn recovered_codec_publication_custody(&self) -> Option<Arc<RecoveredCodecPublicationCustody>> {
        self.recovered_codec_publication_custody()
    }

    fn home_step(&self) -> Option<Arc<AdmittedRelationalHomeProgramStep>> {
        self.home_step()
    }

    fn home_index_definitions(&self) -> Option<Arc<PreparedRelationalIndexDefinitions>> {
        self.home_index_definitions()
    }

    fn covers_home_definitions(
        &self,
        definitions: &PreparedRelationalIndexDefinitions,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, worth_execution::WorkCeilingDenial> {
        self.covers_home_definitions(definitions, execution)
    }

    fn indexes_already_prepared(&self) -> bool {
        self.indexes_already_prepared()
    }

    fn attach_prepared_indexes(
        &mut self,
        prepared: worth_relational_indexes::internal_port::indexes::authority::maintenance::candidate_publication::PreparedCandidateIndexPublication,
    ) -> Result<(), contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind> {
        self.attach_prepared_indexes(prepared)
    }
}

#[doc(hidden)]
pub trait DiscardedRelationalCommitCandidateOwnerPort: Sized {
    fn from_candidate(candidate: &PreparedRelationalCommitCandidate) -> Self;
}

impl DiscardedRelationalCommitCandidateOwnerPort for DiscardedRelationalCommitCandidate {
    fn from_candidate(candidate: &PreparedRelationalCommitCandidate) -> Self {
        Self::from_candidate(candidate)
    }
}
