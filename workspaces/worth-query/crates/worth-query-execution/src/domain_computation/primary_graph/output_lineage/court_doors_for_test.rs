//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn by_source_for_test(
        &self,
    ) -> &BTreeMap<
        SemanticSource,
        BTreeMap<
            worth_runtime_world::facade::ProductBranchIncarnation,
            BTreeMap<u64, RecordedGeneration>,
        >,
    > {
        &self.by_source
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn by_source_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<
        SemanticSource,
        BTreeMap<
            worth_runtime_world::facade::ProductBranchIncarnation,
            BTreeMap<u64, RecordedGeneration>,
        >,
    > {
        &mut self.by_source
    }
}

impl ProductCoordinate {
    #[doc(hidden)]
    pub fn new_for_test(
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
    ) -> Self {
        Self {
            occurrence,
            generation,
        }
    }
}

impl WorthQueryCurrentOutputFamilyResolution {
    #[doc(hidden)]
    pub fn ambiguous_publication_for_test(&self) -> &bool {
        &self.ambiguous_publication
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn binding_identities_for_test(
        &self,
    ) -> &super::super::output_binding_identity::OutputBindingInventory {
        &self.binding_identities
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn cancelled_slots_for_test(&self) -> &Option<Box<prepared_slot::CancelledLineageSlot>> {
        &self.cancelled_slots
    }
}

impl WorthQueryCurrentOutputFamilyResolution {
    #[doc(hidden)]
    pub fn candidates_for_test(&self) -> &Vec<WorthQueryCurrentOutputCandidate> {
        &self.candidates
    }
}

impl WorthQueryPriorOutputBindingResolution {
    #[doc(hidden)]
    pub fn correspondence_for_test(
        &self,
    ) -> &Option<Arc<WorthQueryApplicationOutputCorrespondence>> {
        &self.correspondence
    }
}

impl ProductCoordinate {
    #[doc(hidden)]
    pub fn generation_for_test(&self) -> &u64 {
        &self.generation
    }
}

impl ProductCoordinate {
    #[doc(hidden)]
    pub fn occurrence_for_test(&self) -> &worth_runtime_world::facade::ProductBranchIncarnation {
        &self.occurrence
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn origins_for_test(
        &self,
    ) -> &BTreeMap<worth_runtime_world::facade::ProductBranchIncarnation, ProductCoordinate> {
        &self.origins
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn output_families_for_test(
        &self,
    ) -> &BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>> {
        &self.output_families
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn partition_index_for_test(&self) -> &partition_index::OutputPartitionIndex {
        &self.partition_index
    }
}

impl ProductCoordinate {
    #[doc(hidden)]
    pub fn generation_mut_for_test(&mut self) -> &mut u64 {
        &mut self.generation
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn partition_index_mut_for_test(&mut self) -> &mut partition_index::OutputPartitionIndex {
        &mut self.partition_index
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn install_output_families_for_test(
        &mut self,
        families: BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>>,
    ) {
        Self::install_output_families(self, families)
    }
}

impl WorthQueryCurrentOutputCandidate {
    #[doc(hidden)]
    pub fn correspondence_for_test(&self) -> &Arc<WorthQueryApplicationOutputCorrespondence> {
        &self.correspondence
    }
}

impl WorthQueryCurrentOutputCandidate {
    #[doc(hidden)]
    pub fn settlement_identity_for_test(&self) -> &Arc<RecordedSettlementIdentity> {
        &self.settlement_identity
    }
}

impl WorthQueryExactRecordedOutput {
    #[doc(hidden)]
    pub fn source_identity_for_test(&self) -> &RecordedSourceIdentity {
        &self.source_identity
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn origins_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<worth_runtime_world::facade::ProductBranchIncarnation, ProductCoordinate>
    {
        &mut self.origins
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn output_families_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>> {
        &mut self.output_families
    }
}

impl WorthQueryCurrentOutputCandidate {
    #[doc(hidden)]
    pub fn consumed_outputs_for_test(
        &self,
    ) -> &Arc<[super::super::invariant_projection::ConsumedOutputEvidence]> {
        &self.consumed_outputs
    }
}

impl WorthQueryCurrentOutputCandidate {
    #[doc(hidden)]
    pub fn observed_source_facts_for_test(&self) -> &RetainedSourceFacts {
        &self.observed_source_facts
    }
}

impl WorthQueryCurrentOutputCandidate {
    #[doc(hidden)]
    pub fn verification_requirement_for_test(
        &self,
    ) -> &Option<invalidation::FullVerificationReason> {
        &self.verification_requirement
    }
}

impl WorthQueryExactRecordedOutput {
    #[doc(hidden)]
    pub fn correspondence_for_test(&self) -> &Arc<WorthQueryApplicationOutputCorrespondence> {
        &self.correspondence
    }
}

impl WorthQueryExactRecordedOutput {
    #[doc(hidden)]
    pub fn idempotency_key_identity_for_test(&self) -> &[u8; 32] {
        &self.idempotency_key_identity
    }
}

impl WorthQueryExactRecordedOutput {
    #[doc(hidden)]
    pub fn observed_source_facts_for_test(&self) -> &RetainedSourceFacts {
        &self.observed_source_facts
    }
}

impl WorthQueryExactRecordedOutput {
    #[doc(hidden)]
    pub fn producer_dependency_identity_for_test(&self) -> &Option<[u8; 32]> {
        &self.producer_dependency_identity
    }
}

impl WorthQueryExactRecordedOutput {
    #[doc(hidden)]
    pub fn producing_implementation_for_test(
        &self,
    ) -> &Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation> {
        &self.producing_implementation
    }
}

impl WorthQueryExactRecordedOutput {
    #[doc(hidden)]
    pub fn runtime_authority_for_test(&self) -> &u64 {
        &self.runtime_authority
    }
}

impl WorthQueryExactRecordedOutput {
    #[doc(hidden)]
    pub fn schema_for_test(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
}

impl WorthQueryExactRecordedOutput {
    #[doc(hidden)]
    pub fn source_partition_identity_for_test(&self) -> &[u8; 32] {
        &self.source_partition_identity
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn cancelled_slots_mut_for_test(
        &mut self,
    ) -> &mut Option<Box<prepared_slot::CancelledLineageSlot>> {
        &mut self.cancelled_slots
    }
}

impl WorthQueryCurrentOutputFamilyResolution {
    #[doc(hidden)]
    pub fn candidates_mut_for_test(&mut self) -> &mut Vec<WorthQueryCurrentOutputCandidate> {
        &mut self.candidates
    }
}
