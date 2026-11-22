//! Crate-boundary port exposing owner-private `WorthQueryCompleteApplicationReadSet`
//! internals strictly to upper execution split crates. Never exported on
//! `worth-query-execution::facade` or Host.

use std::marker::PhantomData;
use std::sync::Arc;

use crate::domain_computation::primary_graph::application_attempt::read_set::{
    CompletedFactProjection, CompletedOriginalRead, CompletedReadScopes,
    MutationHandlerBindingProof, SealedComputationFacts, WorkflowOperationBindingProof,
    WorthQueryCompleteApplicationReadSet,
};
use crate::domain_computation::primary_graph::application_attempt::signed_creation::SignedCreationState;
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_program::{
    WorkflowBirthBindingProof, WorkflowSuccessionBindingProof,
};
use crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::LocalActionSettlementProof;
use crate::domain_computation::primary_graph::application_attempt::{
    CompletedFactRoutes, FactContent, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationSnapshotLease,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::operation_body::emission::BodyExternalEffectSelection;
use crate::domain_computation::primary_graph::output_lineage::invalidation::PostconditionCarry;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;

/// Unpacked internal fields of `WorthQueryCompleteApplicationReadSet` for upper
/// execution split crates only.
pub struct CompleteReadSetParts<Schema, Operation, Input, Scope, Phase> {
    pub admission: Box<WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>>,
    pub lease: WorthQueryApplicationSnapshotLease,
    pub application_projection: Option<Arc<CompletedFactProjection>>,
    pub original_read: Option<Arc<CompletedOriginalRead>>,
    pub installed_read_scopes: CompletedReadScopes,
    pub facts: FactPayload,
    pub content: Option<FactContent>,
    pub effect_routes: CompletedFactRoutes,
    pub postcondition_carry: Option<PostconditionCarry>,
    pub consumed_outputs: Vec<ConsumedOutputEvidence>,
    pub computation_facts: Option<SealedComputationFacts>,
    pub computation_fact_ordinals: Arc<[usize]>,
    pub workflow_authority_binding: Option<WorkflowOperationBindingProof>,
    pub workflow_birth_binding: Option<WorkflowBirthBindingProof>,
    pub workflow_succession_binding: Option<WorkflowSuccessionBindingProof>,
    pub mutation_handler_binding: Option<MutationHandlerBindingProof>,
    pub local_action_settlement: Option<LocalActionSettlementProof>,
    pub body_external_emission: Option<BodyExternalEffectSelection>,
    pub signed_creation: Option<SignedCreationState<Schema>>,
    pub workflow_deadline: Option<u64>,
    pub new_commit_refusal: Option<WorthQueryApplicationAttemptDenial>,
    pub _phase: PhantomData<fn() -> Phase>,
    pub _admission_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub trait CompleteReadSetInternalPort<Schema, Operation, Input, Scope, Phase>: Sized {
    fn from_parts(parts: CompleteReadSetParts<Schema, Operation, Input, Scope, Phase>) -> Self;

    fn into_parts(self) -> CompleteReadSetParts<Schema, Operation, Input, Scope, Phase>;

    fn admission(&self)
        -> &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>;

    fn admission_mut(
        &mut self,
    ) -> &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>;

    fn lease(&self) -> &WorthQueryApplicationSnapshotLease;

    fn lease_mut(&mut self) -> &mut WorthQueryApplicationSnapshotLease;

    fn installed_read_scopes(&self) -> &CompletedReadScopes;

    fn original_read(&self) -> Option<&Arc<CompletedOriginalRead>>;

    fn facts(&self) -> &FactPayload;

    fn replace_facts_and_routes(&mut self, facts: FactPayload, routes: CompletedFactRoutes);

    fn content(&self) -> Option<&FactContent>;

    fn effect_routes(&self) -> &CompletedFactRoutes;

    fn postcondition_carry(&self) -> Option<&PostconditionCarry>;

    fn consumed_outputs(&self) -> &[ConsumedOutputEvidence];

    fn consumed_outputs_mut(&mut self) -> &mut Vec<ConsumedOutputEvidence>;

    fn computation_facts(&self) -> Option<&SealedComputationFacts>;

    fn computation_fact_ordinals(&self) -> &Arc<[usize]>;

    fn workflow_authority_binding(&self) -> Option<&WorkflowOperationBindingProof>;

    fn workflow_birth_binding(&self) -> Option<&WorkflowBirthBindingProof>;

    fn workflow_succession_binding(&self) -> Option<&WorkflowSuccessionBindingProof>;

    fn mutation_handler_binding(&self) -> Option<&MutationHandlerBindingProof>;

    fn mutation_handler_binding_mut(&mut self) -> &mut Option<MutationHandlerBindingProof>;

    fn local_action_settlement(&self) -> Option<&LocalActionSettlementProof>;

    fn body_external_emission(&self) -> Option<&BodyExternalEffectSelection>;

    fn signed_creation(&self) -> Option<&SignedCreationState<Schema>>;

    fn signed_creation_mut(&mut self) -> &mut Option<SignedCreationState<Schema>>;

    fn workflow_deadline(&self) -> Option<u64>;

    fn set_workflow_deadline(&mut self, deadline: Option<u64>);

    fn new_commit_refusal(&self) -> Option<&WorthQueryApplicationAttemptDenial>;

    fn set_new_commit_refusal(&mut self, refusal: Option<WorthQueryApplicationAttemptDenial>);

    fn set_local_action_settlement(&mut self, proof: Option<LocalActionSettlementProof>);

    fn set_workflow_authority_binding(&mut self, proof: Option<WorkflowOperationBindingProof>);

    fn set_body_external_emission(&mut self, selection: Option<BodyExternalEffectSelection>);
}

impl<Schema, Operation, Input, Scope, Phase>
    CompleteReadSetInternalPort<Schema, Operation, Input, Scope, Phase>
    for WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>
{
    fn from_parts(parts: CompleteReadSetParts<Schema, Operation, Input, Scope, Phase>) -> Self {
        Self {
            admission: parts.admission,
            lease: parts.lease,
            application_projection: parts.application_projection,
            original_read: parts.original_read,
            installed_read_scopes: parts.installed_read_scopes,
            facts: parts.facts,
            content: parts.content,
            effect_routes: parts.effect_routes,
            postcondition_carry: parts.postcondition_carry,
            consumed_outputs: parts.consumed_outputs,
            computation_facts: parts.computation_facts,
            computation_fact_ordinals: parts.computation_fact_ordinals,
            workflow_authority_binding: parts.workflow_authority_binding,
            workflow_birth_binding: parts.workflow_birth_binding,
            workflow_succession_binding: parts.workflow_succession_binding,
            mutation_handler_binding: parts.mutation_handler_binding,
            local_action_settlement: parts.local_action_settlement,
            body_external_emission: parts.body_external_emission,
            signed_creation: parts.signed_creation,
            workflow_deadline: parts.workflow_deadline,
            new_commit_refusal: parts.new_commit_refusal,
            _phase: parts._phase,
            _admission_memory: parts._admission_memory,
        }
    }

    fn into_parts(self) -> CompleteReadSetParts<Schema, Operation, Input, Scope, Phase> {
        CompleteReadSetParts {
            admission: self.admission,
            lease: self.lease,
            application_projection: self.application_projection,
            original_read: self.original_read,
            installed_read_scopes: self.installed_read_scopes,
            facts: self.facts,
            content: self.content,
            effect_routes: self.effect_routes,
            postcondition_carry: self.postcondition_carry,
            consumed_outputs: self.consumed_outputs,
            computation_facts: self.computation_facts,
            computation_fact_ordinals: self.computation_fact_ordinals,
            workflow_authority_binding: self.workflow_authority_binding,
            workflow_birth_binding: self.workflow_birth_binding,
            workflow_succession_binding: self.workflow_succession_binding,
            mutation_handler_binding: self.mutation_handler_binding,
            local_action_settlement: self.local_action_settlement,
            body_external_emission: self.body_external_emission,
            signed_creation: self.signed_creation,
            workflow_deadline: self.workflow_deadline,
            new_commit_refusal: self.new_commit_refusal,
            _phase: self._phase,
            _admission_memory: self._admission_memory,
        }
    }

    fn admission(
        &self,
    ) -> &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope> {
        &self.admission
    }

    fn admission_mut(
        &mut self,
    ) -> &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope> {
        &mut self.admission
    }

    fn lease(&self) -> &WorthQueryApplicationSnapshotLease {
        &self.lease
    }

    fn lease_mut(&mut self) -> &mut WorthQueryApplicationSnapshotLease {
        &mut self.lease
    }

    fn installed_read_scopes(&self) -> &CompletedReadScopes {
        &self.installed_read_scopes
    }

    fn original_read(&self) -> Option<&Arc<CompletedOriginalRead>> {
        self.original_read.as_ref()
    }

    fn facts(&self) -> &FactPayload {
        &self.facts
    }

    fn replace_facts_and_routes(&mut self, facts: FactPayload, routes: CompletedFactRoutes) {
        self.facts = facts;
        self.effect_routes = routes;
    }

    fn content(&self) -> Option<&FactContent> {
        self.content.as_ref()
    }

    fn effect_routes(&self) -> &CompletedFactRoutes {
        &self.effect_routes
    }

    fn postcondition_carry(&self) -> Option<&PostconditionCarry> {
        self.postcondition_carry.as_ref()
    }

    fn consumed_outputs(&self) -> &[ConsumedOutputEvidence] {
        &self.consumed_outputs
    }

    fn consumed_outputs_mut(&mut self) -> &mut Vec<ConsumedOutputEvidence> {
        &mut self.consumed_outputs
    }

    fn computation_facts(&self) -> Option<&SealedComputationFacts> {
        self.computation_facts.as_ref()
    }

    fn computation_fact_ordinals(&self) -> &Arc<[usize]> {
        &self.computation_fact_ordinals
    }

    fn workflow_authority_binding(&self) -> Option<&WorkflowOperationBindingProof> {
        self.workflow_authority_binding.as_ref()
    }

    fn workflow_birth_binding(&self) -> Option<&WorkflowBirthBindingProof> {
        self.workflow_birth_binding.as_ref()
    }

    fn workflow_succession_binding(&self) -> Option<&WorkflowSuccessionBindingProof> {
        self.workflow_succession_binding.as_ref()
    }

    fn mutation_handler_binding(&self) -> Option<&MutationHandlerBindingProof> {
        self.mutation_handler_binding.as_ref()
    }

    fn mutation_handler_binding_mut(&mut self) -> &mut Option<MutationHandlerBindingProof> {
        &mut self.mutation_handler_binding
    }

    fn local_action_settlement(&self) -> Option<&LocalActionSettlementProof> {
        self.local_action_settlement.as_ref()
    }

    fn body_external_emission(&self) -> Option<&BodyExternalEffectSelection> {
        self.body_external_emission.as_ref()
    }

    fn signed_creation(&self) -> Option<&SignedCreationState<Schema>> {
        self.signed_creation.as_ref()
    }

    fn signed_creation_mut(&mut self) -> &mut Option<SignedCreationState<Schema>> {
        &mut self.signed_creation
    }

    fn workflow_deadline(&self) -> Option<u64> {
        self.workflow_deadline
    }

    fn set_workflow_deadline(&mut self, deadline: Option<u64>) {
        self.workflow_deadline = deadline;
    }

    fn new_commit_refusal(&self) -> Option<&WorthQueryApplicationAttemptDenial> {
        self.new_commit_refusal.as_ref()
    }

    fn set_new_commit_refusal(&mut self, refusal: Option<WorthQueryApplicationAttemptDenial>) {
        self.new_commit_refusal = refusal;
    }

    fn set_local_action_settlement(&mut self, proof: Option<LocalActionSettlementProof>) {
        self.local_action_settlement = proof;
    }

    fn set_workflow_authority_binding(&mut self, proof: Option<WorkflowOperationBindingProof>) {
        self.workflow_authority_binding = proof;
    }

    fn set_body_external_emission(&mut self, selection: Option<BodyExternalEffectSelection>) {
        self.body_external_emission = selection;
    }
}
