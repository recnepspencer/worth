//! One actual producer field comparison as the evidence settlement and readiness delivery
//! retain. The producer's installed output scope is read through `ProducerOutputFieldScope`;
//! the comparison itself is performed and constructed by the producer that owns the scope.
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::AspectContractRevision;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;
use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::RelationalFieldRevision;
use worth_runtime_bridge::facade::BridgeExactOutputComparison;

/// The installed output field one producer compares: its locus, the aspect contract revision
/// it was sealed against, and its comparison meaning. Retained evidence reads nothing else.
pub(in crate::domain_computation::primary_graph) trait ProducerOutputFieldScope {
    fn locus(&self) -> &ApplicationFieldBindingLocus;
    fn contract_revision(&self) -> AspectContractRevision;
    fn meaning(&self) -> [u8; 32];
}

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

pub(in crate::domain_computation::primary_graph) struct ProducerOutputEvaluation {
    field: Arc<dyn ProducerOutputFieldScope + Send + Sync>,
    role: Arc<str>,
    pub(in crate::domain_computation::primary_graph) state: ProducerOutputEvaluationState,
    _memory: Option<ExecutionMemoryReservation>,
}

pub(in crate::domain_computation::primary_graph) enum ProducerOutputEvaluationState {
    Ineligible(WorthQueryProducerEvaluationIneligibility),
    CaptureUnavailable(
        crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure,
    ),
    Compared(ComparedProducerOutput),
}

pub(in crate::domain_computation::primary_graph) struct ComparedProducerOutput {
    scope: Arc<dyn ProducerOutputFieldScope + Send + Sync>,
    pub(in crate::domain_computation::primary_graph) role: usize,
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) kind: KindId,
    pub(in crate::domain_computation::primary_graph) revision: RelationalFieldRevision,
    pub(in crate::domain_computation::primary_graph) performed_basis:
        RelationalBranchBasisDescriptor,
    pub(in crate::domain_computation::primary_graph) comparison: BridgeExactOutputComparison,
}

impl ComparedProducerOutput {
    /// The producer's completed comparison of its installed `scope`.
    pub(in crate::domain_computation::primary_graph) fn new(
        scope: Arc<dyn ProducerOutputFieldScope + Send + Sync>,
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
    pub(in crate::domain_computation::primary_graph) fn field(
        &self,
    ) -> &ApplicationFieldBindingLocus {
        self.scope.locus()
    }
    pub(in crate::domain_computation::primary_graph) fn contract_revision(
        &self,
    ) -> AspectContractRevision {
        self.scope.contract_revision()
    }
    pub(in crate::domain_computation::primary_graph) fn field_meaning(&self) -> [u8; 32] {
        self.scope.meaning()
    }
}

impl ProducerOutputEvaluation {
    /// One completed evaluation of `field` on `role`, holding the memory reserved for it.
    pub(in crate::domain_computation::primary_graph) fn new(
        field: Arc<dyn ProducerOutputFieldScope + Send + Sync>,
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
    pub(in crate::domain_computation::primary_graph) fn field(
        &self,
    ) -> &ApplicationFieldBindingLocus {
        self.field.locus()
    }
    pub(in crate::domain_computation::primary_graph) fn role(&self) -> &str {
        &self.role
    }
}
