//! Exact original source/input and producer/output joins, after fresh diagnostics.
use super::WorthQueryApplicationInspectionTarget;
mod evaluation;
mod output_refusal;
pub use evaluation::{
    WorthQueryApplicationProducerEvaluation, WorthQueryProducerEvaluationDecision,
    WorthQueryProducerEvaluationDenial,
};
#[derive(Clone, Copy)]
pub(super) enum SourceInspectionPurpose {
    Creation,
    Evaluation,
}
pub(super) enum ProducerExplanation {
    Creation(WorthQueryApplicationProducerConnection),
    Evaluation(WorthQueryApplicationProducerEvaluation),
}

mod overlap;
mod publication;
mod source;
use crate::domain_computation::{
    authorization::application_disclosure::DiagnosticComparisonAdmission,
    primary_graph::{
        admitted_capture::{run_retained_stage, CaptureStop},
        application_checkpoint::facts::FactPayload,
        application_output_demand::OriginalSettledProducerSource,
        source_disclosure::OriginalSourceInspectionDisclosure,
        WorthQueryApplicationBasisSelectionIdentity, WorthQueryApplicationCommitReceipt,
        WorthQueryApplicationCommittedChanges, WorthQueryApplicationOutputCorrespondence,
    },
};
pub use output_refusal::WorthQueryProducerOutputConnectionDenial;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure};
use worth_relational::facade::history::RelationalCommitReceipt;

/// Original retained evidence could not establish this requested connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerConnectionDenial {
    Evaluation(WorthQueryProducerEvaluationDenial),
    ForeignSource,
    SourceDisclosureUnavailable,
    SourceDisclosureMismatch,
    ProducerUnavailable,
    PerformedInputsUnavailable,
    SourceInputUnconnected,
    OutputUnconnected,
}

/// Only the connection owner can attach output evidence after diagnostic admission.
pub(super) enum ConnectionRefusal {
    General(WorthQueryProducerConnectionDenial),
    Output(WorthQueryProducerOutputConnectionDenial),
}
impl From<WorthQueryProducerConnectionDenial> for ConnectionRefusal {
    fn from(cause: WorthQueryProducerConnectionDenial) -> Self {
        Self::General(cause)
    }
}
impl ConnectionRefusal {
    pub(super) fn kind(&self) -> WorthQueryProducerConnectionDenial {
        match self {
            Self::General(cause) => *cause,
            Self::Output(_) => WorthQueryProducerConnectionDenial::OutputUnconnected,
        }
    }
}

/// A source field was actually read by this producer, whose declared output
/// field was actually read by the query. This grants no execution authority.
pub struct WorthQueryApplicationProducerConnection {
    source: WorthQueryApplicationCommittedChanges,
    producer: WorthQueryApplicationCommittedChanges,
    output_publication: WorthQueryApplicationCommittedChanges,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    role: usize,
    _inputs: FactPayload,
    _disclosure: Arc<OriginalSourceInspectionDisclosure>,
    native: worth_runtime_bridge::facade::BridgeConditionalCausalObservation,
    output_native: worth_runtime_bridge::facade::BridgeConditionalCausalObservation,
}

impl WorthQueryApplicationProducerConnection {
    pub(super) fn native_observation(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeConditionalCausalObservation {
        &self.native
    }

    pub(super) fn output_native_observation(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeConditionalCausalObservation {
        &self.output_native
    }

    pub fn source_commit(&self) -> &RelationalCommitReceipt {
        self.source.commit_reference()
    }
    /// Current settled producer publication; this remains the currentness owner.
    pub fn producer_commit(&self) -> &RelationalCommitReceipt {
        self.producer.commit_reference()
    }
    /// The actual publication joined to the queried field's Native revision.
    /// It may precede the current settlement returned by `producer_commit()`.
    pub fn output_publication_commit(&self) -> &RelationalCommitReceipt {
        self.output_publication.commit_reference()
    }
    /// Visits the original declared role roster up to the connected role.
    pub fn output_role(&self) -> &str {
        self.correspondence
            .native_witness_roles()
            .nth(self.role)
            .expect("connected original role")
            .0
    }
    /// Current settlement's actual readiness, distinct from execution or suppression.
    pub fn readiness_decision(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision {
        crate::domain_computation::primary_graph::conditional_operation::classify_bridge_signal_observation(&self.native)
    }
    pub fn has_performed_invalidation(&self) -> bool {
        self.native.performed_signal_invalidation().is_some()
    }
}

pub(super) fn connect<Schema: worth_query_installation::facade::ApplicationSchema>(
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    execution: &ExecutionRequest<'_, '_>,
    target: &WorthQueryApplicationInspectionTarget,
    source: &WorthQueryApplicationCommitReceipt,
    diagnostic: &DiagnosticComparisonAdmission<'_>,
    source_diagnostic: Option<&DiagnosticComparisonAdmission<'_>>,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<WorthQueryApplicationProducerConnection, CaptureStop<ConnectionRefusal>> {
    use WorthQueryProducerConnectionDenial as Denial;
    run_retained_stage(execution, memory, |context, _| {
        let disclosure = source::validate(context, target, source)?;
        let original = target
            .original()
            .ok_or(MapKernelFailure::Domain(Denial::ForeignSource.into()))?;
        let supplier = original
            .source
            .original_settled_producer()
            .ok_or(MapKernelFailure::Domain(Denial::ProducerUnavailable.into()))?;
        if !matches!(&original.source.selection,
            WorthQueryApplicationBasisSelectionIdentity::Product(read) if supplier.read() == read)
        {
            return Err(MapKernelFailure::Domain(Denial::ProducerUnavailable.into()));
        }
        let OriginalSettledProducerSource::Committed {
            changes,
            inputs,
            conditional: Some(native),
            ..
        } = supplier.source()
        else {
            return Err(MapKernelFailure::Domain(Denial::ProducerUnavailable.into()));
        };
        let inputs = inputs.as_ref().ok_or(MapKernelFailure::Domain(
            Denial::PerformedInputsUnavailable.into(),
        ))?;
        let observed = overlap::ObservationConnection {
            original,
            diagnostic,
            layout: &runtime.primary_provider.graph.layout,
            contracts: runtime.installed_schema().native_contracts(),
        };
        source::require_input(
            &observed,
            context,
            target,
            source.committed_changes(),
            inputs,
            source_diagnostic,
        )?;
        let output = publication::select(
            &observed,
            context,
            target,
            source,
            supplier,
            source_diagnostic,
        )?;
        Ok(WorthQueryApplicationProducerConnection {
            source: source.committed_changes().clone(),
            producer: changes.clone(),
            output_publication: output.changes,
            correspondence: supplier.retain_correspondence(),
            role: output.role,
            _inputs: output.inputs,
            _disclosure: disclosure,
            native: native.clone(),
            output_native: output.native,
        })
    })
}

pub(super) fn explain<Schema: worth_query_installation::facade::ApplicationSchema>(
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    execution: &ExecutionRequest<'_, '_>,
    target: &WorthQueryApplicationInspectionTarget,
    source: &WorthQueryApplicationCommitReceipt,
    diagnostic: &DiagnosticComparisonAdmission<'_>,
    source_diagnostic: Option<&DiagnosticComparisonAdmission<'_>>,
    memory: &mut Option<ExecutionMemoryReservation>,
    purpose: SourceInspectionPurpose,
) -> Result<ProducerExplanation, CaptureStop<ConnectionRefusal>> {
    match purpose {
        SourceInspectionPurpose::Creation => connect(
            runtime,
            execution,
            target,
            source,
            diagnostic,
            source_diagnostic,
            memory,
        )
        .map(ProducerExplanation::Creation),
        SourceInspectionPurpose::Evaluation => evaluation::evaluate(
            runtime,
            execution,
            target,
            source,
            diagnostic,
            source_diagnostic,
            memory,
        )
        .map(ProducerExplanation::Evaluation),
    }
}
