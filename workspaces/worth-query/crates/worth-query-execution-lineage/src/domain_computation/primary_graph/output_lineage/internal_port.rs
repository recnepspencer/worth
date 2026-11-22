//! Crate-boundary internal port for mutable `WorthQueryApplicationOutputLineage`
//! operations. Never exported on `worth-query-execution::facade` or Host.

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::restoration::RestoredRecord;
use super::{
    OutputFamilyIdentity, RecordedSourceIdentity, RetainedSourceFacts, SealedNativeOutputWitness,
    WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence;
use crate::equality_type_id::EqualityTypeId as TypeId;

pub trait ApplicationOutputLineageInternalPort {
    fn install_output_families(
        &mut self,
        families: BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>>,
    );

    fn register_fork(
        &mut self,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        destination: &worth_runtime_world::facade::ProductBranchObservation,
    );

    #[allow(clippy::too_many_arguments)]
    fn record_restoration(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        source_identity: RecordedSourceIdentity,
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
        observed_source_facts: RetainedSourceFacts,
        verified_witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
        checkpoint: Option<
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
        >,
        producing_implementation: Option<
            worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
        >,
    ) -> Option<RestoredRecord>;

    fn release_occurrence(
        &mut self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    );
}

impl ApplicationOutputLineageInternalPort for WorthQueryApplicationOutputLineage {
    fn install_output_families(
        &mut self,
        families: BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>>,
    ) {
        Self::install_output_families(self, families)
    }

    fn register_fork(
        &mut self,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        destination: &worth_runtime_world::facade::ProductBranchObservation,
    ) {
        Self::register_fork(self, source, destination)
    }

    fn record_restoration(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        source_identity: RecordedSourceIdentity,
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
        observed_source_facts: RetainedSourceFacts,
        verified_witness: Option<Arc<OnceLock<SealedNativeOutputWitness>>>,
        checkpoint: Option<
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
        >,
        producing_implementation: Option<
            worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
        >,
    ) -> Option<RestoredRecord> {
        Self::record_restoration(
            self,
            output_binding,
            runtime_authority,
            schema,
            scope,
            observation,
            correspondence,
            source_identity,
            source_partition_identity,
            producer_dependency_identity,
            idempotency_key_identity,
            observed_source_facts,
            verified_witness,
            checkpoint,
            producing_implementation,
        )
    }

    fn release_occurrence(
        &mut self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        Self::release_occurrence(self, occurrence)
    }
}
