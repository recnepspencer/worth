//! Crate-boundary ports for invariant mutation targets, output demand
//! settlements, and observed sources. Never exported on
//! `worth-query-execution::facade` or Host.

use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::CanonicalDigestId;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification;

use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_attempt::{
    ManagedSourceRead, SourceReadAuthorization, WorthQueryApplicationObservedFact,
    WorthQueryApplicationOutputCorrespondence,
};
use crate::domain_computation::primary_graph::application_output_demand::settlement::{
    SettlementProducerIdentity, SettlementReceiptCustody, WorthQueryOutputDemandSettlement,
    WorthQueryOutputSettlementPosture,
};
use crate::domain_computation::primary_graph::application_output_demand::{
    WorthQueryOutputReadinessDeliveryEvidence, WorthQueryRestoredAcceptedOutput,
};
use crate::domain_computation::primary_graph::application_query::observed_source::fact_conversion::{
    PreparedSourceFactMaterialization, SourceFactExpansionControl,
};
use crate::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaning;
use crate::domain_computation::primary_graph::application_query::observed_source::{
    RetainedExpressionContext, WorthQueryBoundSourceExpectation, WorthQueryObservedScopeSelector,
    WorthQueryObservedSource, WorthQueryRetainedObservedParameters,
    WorthQuerySourceExpectationDenial, WorthQuerySourceExpectationKind,
};
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationReadObservation;
use crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisSelectionIdentity, WorthQueryInvariantMutationTarget,
};

pub trait InvariantMutationTargetInternalPort<Schema, Entity> {
    fn entity_id(&self) -> EntityId;
    fn entity(&self) -> &Arc<str>;
    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity;
    fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity;
    fn admission_identity(&self) -> WorthQueryOperationAdmissionIdentity;
    fn phase(&self) -> Option<RelationalAppliedCandidateReadQualification>;
    fn fragment_schema<TargetSchema>(&self) -> WorthQueryInvariantMutationTarget<TargetSchema, ()>;
}

impl<Schema, Entity> InvariantMutationTargetInternalPort<Schema, Entity>
    for WorthQueryInvariantMutationTarget<Schema, Entity>
{
    fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    fn entity(&self) -> &Arc<str> {
        &self.entity
    }

    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    fn admission_identity(&self) -> WorthQueryOperationAdmissionIdentity {
        self.admission_identity
    }

    fn phase(&self) -> Option<RelationalAppliedCandidateReadQualification> {
        self.phase
    }

    fn fragment_schema<TargetSchema>(&self) -> WorthQueryInvariantMutationTarget<TargetSchema, ()> {
        Self::fragment_schema(self)
    }
}

pub struct OutputDemandSettlementParts {
    pub original_delivery_memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub posture: WorthQueryOutputSettlementPosture,
    pub runtime_authority: u64,
    pub schema_binding: ApplicationSchemaBindingIdentity,
    pub producer_identity: SettlementProducerIdentity,
    pub output_family_identity: String,
    pub receipt: Option<SettlementReceiptCustody>,
    pub stable: Option<PublishedStableLineage>,
    pub output_correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub restored_source: Option<WorthQueryRestoredAcceptedOutput>,
    pub readiness_delivery: Option<WorthQueryOutputReadinessDeliveryEvidence>,
    pub producer_contacts_in_this_demand: usize,
    pub observation: Arc<WorthQueryApplicationReadObservation>,
}

pub trait OutputDemandSettlementInternalPort: Sized {
    fn from_parts(parts: OutputDemandSettlementParts) -> Self;
    fn posture(&self) -> WorthQueryOutputSettlementPosture;
    fn restored_source(&self) -> Option<&WorthQueryRestoredAcceptedOutput>;
    fn stable(&self) -> Option<&PublishedStableLineage>;
    fn observation(&self) -> &Arc<WorthQueryApplicationReadObservation>;
    fn set_original_delivery_memory(
        &mut self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    );
}

impl OutputDemandSettlementInternalPort for WorthQueryOutputDemandSettlement {
    fn from_parts(parts: OutputDemandSettlementParts) -> Self {
        Self {
            _original_delivery_memory: parts.original_delivery_memory,
            posture: parts.posture,
            runtime_authority: parts.runtime_authority,
            schema_binding: parts.schema_binding,
            producer_identity: parts.producer_identity,
            output_family_identity: parts.output_family_identity,
            receipt: parts.receipt,
            stable: parts.stable,
            output_correspondence: parts.output_correspondence,
            restored_source: parts.restored_source,
            readiness_delivery: parts.readiness_delivery,
            producer_contacts_in_this_demand: parts.producer_contacts_in_this_demand,
            observation: parts.observation,
        }
    }

    fn posture(&self) -> WorthQueryOutputSettlementPosture {
        self.posture
    }

    fn restored_source(&self) -> Option<&WorthQueryRestoredAcceptedOutput> {
        self.restored_source.as_ref()
    }

    fn stable(&self) -> Option<&PublishedStableLineage> {
        self.stable.as_ref()
    }

    fn observation(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.observation
    }

    fn set_original_delivery_memory(
        &mut self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) {
        self._original_delivery_memory = memory;
    }
}

pub struct ObservedSourceParts<Query> {
    pub expression_context: Option<Arc<RetainedExpressionContext>>,
    pub runtime_authority: u64,
    pub schema_binding: ApplicationSchemaBindingIdentity,
    pub query_identity: WorthQueryInstalledApplicationQueryIdentity,
    pub parameter_binding_identity: CanonicalDigestId,
    pub parameters: Arc<WorthQueryRetainedObservedParameters>,
    pub query_identifier: Arc<String>,
    pub branch: Arc<BranchId>,
    pub selection: WorthQueryApplicationBasisSelectionIdentity,
    pub model_root: EntityId,
    pub invocation_source_authorization: Option<Arc<SourceReadAuthorization>>,
    pub invocation_managed_source: Option<Arc<ManagedSourceRead>>,
    pub source_meaning: Arc<WorthQueryObservedSourceMeaning>,
    pub scope_selector: Arc<WorthQueryObservedScopeSelector>,
    pub _marker: PhantomData<fn() -> Query>,
}

pub trait ObservedSourceInternalPort<Query>: Sized {
    fn from_parts(parts: ObservedSourceParts<Query>) -> Self;
    fn source_meaning(&self) -> &Arc<WorthQueryObservedSourceMeaning>;
    fn retained_parameters(&self) -> &Arc<WorthQueryRetainedObservedParameters>;
    fn retained_scope_selector(&self) -> &Arc<WorthQueryObservedScopeSelector>;
    fn query_identity(&self) -> WorthQueryInstalledApplicationQueryIdentity;
    fn query_identifier(&self) -> &Arc<String>;
    fn runtime_authority(&self) -> u64;
    fn schema_binding(&self) -> ApplicationSchemaBindingIdentity;
    fn schema_binding_ref(&self) -> &ApplicationSchemaBindingIdentity;
    fn selection(&self) -> WorthQueryApplicationBasisSelectionIdentity;
    fn model_root(&self) -> EntityId;
    fn parameter_binding_identity(&self) -> CanonicalDigestId;
    fn parameters(&self) -> &Arc<WorthQueryRetainedObservedParameters>;
    fn scope_selector(&self) -> &Arc<WorthQueryObservedScopeSelector>;
    fn branch(&self) -> &Arc<BranchId>;
    fn admit_fact_materialization<'source, 'request, 'authority>(
        &'source self,
        layout: &WorthQueryPrimaryGraphLayout,
        subject: &str,
        control: SourceFactExpansionControl<'request, 'authority>,
    ) -> Result<
        PreparedSourceFactMaterialization<'source, 'request, 'authority>,
        WorthQuerySourceExpectationDenial,
    >;
    fn retained_checkpoint_facts(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Arc<[WorthQueryApplicationObservedFact]>, WorthQuerySourceExpectationDenial>;
}

impl<Query> ObservedSourceInternalPort<Query> for WorthQueryObservedSource<Query> {
    fn from_parts(parts: ObservedSourceParts<Query>) -> Self {
        Self {
            expression_context: parts.expression_context,
            runtime_authority: parts.runtime_authority,
            schema_binding: parts.schema_binding,
            query_identity: parts.query_identity,
            parameter_binding_identity: parts.parameter_binding_identity,
            parameters: parts.parameters,
            query_identifier: parts.query_identifier,
            branch: parts.branch,
            selection: parts.selection,
            model_root: parts.model_root,
            invocation_source_authorization: parts.invocation_source_authorization,
            invocation_managed_source: parts.invocation_managed_source,
            source_meaning: parts.source_meaning,
            scope_selector: parts.scope_selector,
            _marker: parts._marker,
        }
    }

    fn source_meaning(&self) -> &Arc<WorthQueryObservedSourceMeaning> {
        &self.source_meaning
    }

    fn retained_parameters(&self) -> &Arc<WorthQueryRetainedObservedParameters> {
        Self::retained_parameters(self)
    }

    fn retained_scope_selector(&self) -> &Arc<WorthQueryObservedScopeSelector> {
        Self::retained_scope_selector(self)
    }

    fn query_identity(&self) -> WorthQueryInstalledApplicationQueryIdentity {
        self.query_identity.clone()
    }

    fn query_identifier(&self) -> &Arc<String> {
        &self.query_identifier
    }

    fn runtime_authority(&self) -> u64 {
        self.runtime_authority
    }

    fn schema_binding(&self) -> ApplicationSchemaBindingIdentity {
        self.schema_binding.clone()
    }

    fn schema_binding_ref(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    fn selection(&self) -> WorthQueryApplicationBasisSelectionIdentity {
        self.selection.clone()
    }

    fn model_root(&self) -> EntityId {
        self.model_root
    }

    fn parameter_binding_identity(&self) -> CanonicalDigestId {
        self.parameter_binding_identity
    }

    fn parameters(&self) -> &Arc<WorthQueryRetainedObservedParameters> {
        &self.parameters
    }

    fn scope_selector(&self) -> &Arc<WorthQueryObservedScopeSelector> {
        &self.scope_selector
    }

    fn branch(&self) -> &Arc<BranchId> {
        &self.branch
    }

    fn admit_fact_materialization<'source, 'request, 'authority>(
        &'source self,
        layout: &WorthQueryPrimaryGraphLayout,
        subject: &str,
        control: SourceFactExpansionControl<'request, 'authority>,
    ) -> Result<
        PreparedSourceFactMaterialization<'source, 'request, 'authority>,
        WorthQuerySourceExpectationDenial,
    > {
        Self::admit_fact_materialization(self, layout, subject, control)
    }

    fn retained_checkpoint_facts(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Arc<[WorthQueryApplicationObservedFact]>, WorthQuerySourceExpectationDenial> {
        Self::retained_checkpoint_facts(self, layout, request, allocation_policy)
    }
}

pub trait BoundSourceExpectationInternalPort: Sized {
    fn from_parts(
        kind: WorthQuerySourceExpectationKind,
        identity: [u8; 32],
        partition_identity: [u8; 32],
    ) -> Self;
    fn kind(&self) -> WorthQuerySourceExpectationKind;
    fn identity(&self) -> [u8; 32];
    fn partition_identity_bytes(&self) -> [u8; 32];
}

impl BoundSourceExpectationInternalPort for WorthQueryBoundSourceExpectation {
    fn from_parts(
        kind: WorthQuerySourceExpectationKind,
        identity: [u8; 32],
        partition_identity: [u8; 32],
    ) -> Self {
        Self {
            kind,
            identity,
            partition_identity,
        }
    }

    fn kind(&self) -> WorthQuerySourceExpectationKind {
        self.kind
    }

    fn identity(&self) -> [u8; 32] {
        self.identity
    }

    fn partition_identity_bytes(&self) -> [u8; 32] {
        self.partition_identity
    }
}
