//! One actual producer field comparison, completed before Ready and retained through retries.
use super::output_equivalence_contract::InstalledProducerOutputEquivalence;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
use worth_relational::facade::{
    branch::RelationalBranchBasisDescriptor,
    identity::{EntityId, KindId},
    runtime::RelationalFieldRevision,
};
use worth_runtime_bridge::facade::BridgeExactOutputComparison;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerEvaluationIneligibility {
    OutputRoleUnavailable,
    BeforeFieldUnavailable,
    ConflictingBeforeField,
    NativeAfterUnavailable,
    ConflictingNativeAfter,
    NativeAfterAbsent,
    ReadinessPublicationMismatch,
    ReadinessFieldMismatch,
}

pub struct ProducerOutputEvaluation {
    field: Arc<InstalledProducerOutputEquivalence>,
    role: Arc<str>,
    pub state: ProducerOutputEvaluationState,
    _memory: Option<ExecutionMemoryReservation>,
}

pub enum ProducerOutputEvaluationState {
    Ineligible(WorthQueryProducerEvaluationIneligibility),
    CaptureUnavailable(
        #[allow(
            dead_code,
            reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
        )]
        crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure,
    ),
    Compared(ComparedProducerOutput),
}

pub struct ComparedProducerOutput {
    scope: Arc<InstalledProducerOutputEquivalence>,
    pub role: usize,
    pub entity: EntityId,
    pub kind: KindId,
    pub revision: RelationalFieldRevision,
    pub performed_basis: RelationalBranchBasisDescriptor,
    pub comparison: BridgeExactOutputComparison,
}

impl ComparedProducerOutput {
    pub fn new(
        scope: Arc<InstalledProducerOutputEquivalence>,
        role: usize,
        entity: EntityId,
        kind: KindId,
        revision: RelationalFieldRevision,
        performed_basis: RelationalBranchBasisDescriptor,
        comparison: BridgeExactOutputComparison,
    ) -> Self {
        Self {
            scope,
            role,
            entity,
            kind,
            revision,
            performed_basis,
            comparison,
        }
    }

    pub fn field(
        &self,
    ) -> &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus {
        &self.scope.field.binding.locus
    }

    pub fn contract_revision(&self) -> worth_foundational::facade::AspectContractRevision {
        self.scope.field.aspect.revision()
    }

    pub fn field_meaning(&self) -> [u8; 32] {
        self.scope.meaning
    }
}

impl ProducerOutputEvaluation {
    pub fn new(
        field: Arc<InstalledProducerOutputEquivalence>,
        role: Arc<str>,
        state: ProducerOutputEvaluationState,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            field,
            role,
            state,
            _memory: memory,
        }
    }

    pub fn field(
        &self,
    ) -> &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus {
        &self.field.field.binding.locus
    }

    pub fn role(&self) -> &str {
        &self.role
    }
}
