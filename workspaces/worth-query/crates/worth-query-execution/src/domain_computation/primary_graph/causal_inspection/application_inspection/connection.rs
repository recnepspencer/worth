//! Exact original source/input and producer/output joins, after fresh diagnostics.
use super::WorthQueryApplicationInspectionTarget;
mod overlap;
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
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure};
use worth_relational::facade::history::RelationalCommitReceipt;

/// Original retained evidence could not establish this requested connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerConnectionDenial {
    ForeignSource,
    SourceDisclosureUnavailable,
    SourceDisclosureMismatch,
    ProducerUnavailable,
    PerformedInputsUnavailable,
    SourceInputUnconnected,
    OutputUnconnected,
}

/// A source field was actually read by this producer, whose declared output
/// field was actually read by the query. This grants no execution authority.
pub struct WorthQueryApplicationProducerConnection {
    source: WorthQueryApplicationCommittedChanges,
    producer: WorthQueryApplicationCommittedChanges,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    role: usize,
    _inputs: FactPayload,
    _disclosure: Arc<OriginalSourceInspectionDisclosure>,
    native: worth_runtime_bridge::facade::BridgeConditionalCausalObservation,
}

impl WorthQueryApplicationProducerConnection {
    pub(super) fn native_observation(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeConditionalCausalObservation {
        &self.native
    }

    pub fn source_commit(&self) -> &RelationalCommitReceipt {
        self.source.commit_reference()
    }
    pub fn producer_commit(&self) -> &RelationalCommitReceipt {
        self.producer.commit_reference()
    }
    /// Visits the original declared role roster up to the connected role.
    pub fn output_role(&self) -> &str {
        self.correspondence
            .native_witness_roles()
            .nth(self.role)
            .expect("connected original role")
            .0
    }
    /// Actual readiness decision, distinct from producer execution or suppression.
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
) -> Result<WorthQueryApplicationProducerConnection, CaptureStop<WorthQueryProducerConnectionDenial>>
{
    use WorthQueryProducerConnectionDenial as Denial;
    run_retained_stage(execution, memory, |context, _| {
        let disclosure = source::validate(context, target, source)?;
        let original = target
            .original()
            .ok_or(MapKernelFailure::Domain(Denial::ForeignSource))?;
        let supplier = original
            .source
            .original_settled_producer()
            .ok_or(MapKernelFailure::Domain(Denial::ProducerUnavailable))?;
        if !matches!(&original.source.selection,
            WorthQueryApplicationBasisSelectionIdentity::Product(read) if supplier.read() == read)
        {
            return Err(MapKernelFailure::Domain(Denial::ProducerUnavailable));
        }
        let OriginalSettledProducerSource::Committed {
            changes,
            inputs,
            conditional: Some(native),
            ..
        } = supplier.source()
        else {
            return Err(MapKernelFailure::Domain(Denial::ProducerUnavailable));
        };
        let inputs = inputs
            .as_ref()
            .ok_or(MapKernelFailure::Domain(Denial::PerformedInputsUnavailable))?;
        let observed = overlap::ObservationConnection {
            original,
            diagnostic,
            layout: &runtime.primary_provider.graph.layout,
            contracts: runtime.installed_schema().native_contracts(),
        };
        if let Some(source_diagnostic) = source_diagnostic {
            observed.require_independent_input(
                context,
                execution,
                source.committed_changes(),
                inputs,
                source_diagnostic,
                target
                    .connection_source()
                    .and_then(|target| target.source())
                    .ok_or(MapKernelFailure::Domain(Denial::SourceDisclosureMismatch))?
                    .disclosure
                    .scope()
                    .subject(),
            )?;
        } else {
            observed.require_input(context, execution, source.committed_changes(), inputs)?;
        }
        let role = observed.output_role(context, execution, changes, supplier.correspondence())?;
        Ok(WorthQueryApplicationProducerConnection {
            source: source.committed_changes().clone(),
            producer: changes.clone(),
            correspondence: supplier.retain_correspondence(),
            role,
            _inputs: inputs.clone(),
            _disclosure: disclosure,
            native: native.clone(),
        })
    })
}
