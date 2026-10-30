//! A performed evaluation joins the observed revision without claiming to create it.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    ProducerOutputEvaluation, ProducerOutputEvaluationState,
};
use worth_execution::MapKernelContext;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerEvaluationDenial {
    EvaluationUnavailable,
    NotPerformed,
    OriginalCaptureUnavailable,
    Ineligible(crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerEvaluationIneligibility),
    ObservedFieldUnconnected,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerEvaluationDecision {
    Changed,
    Equivalent,
}

/// An independently admitted explanation for the declared output field only.
pub struct WorthQueryApplicationProducerEvaluation {
    source: WorthQueryApplicationCommittedChanges,
    producer: WorthQueryApplicationCommittedChanges,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    evaluation: Arc<ProducerOutputEvaluation>,
    _inputs: FactPayload,
    _disclosure: Arc<OriginalSourceInspectionDisclosure>,
}
impl WorthQueryApplicationProducerEvaluation {
    fn compared(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_contribution::ComparedProducerOutput
    {
        let ProducerOutputEvaluationState::Compared(compared) = &self.evaluation.state else {
            unreachable!("only an actual comparison can issue an admitted evaluation report")
        };
        compared
    }
    pub fn source_commit(&self) -> &RelationalCommitReceipt {
        self.source.commit_reference()
    }
    pub fn producer_commit(&self) -> &RelationalCommitReceipt {
        self.producer.commit_reference()
    }
    pub fn output_role(&self) -> &str {
        self.correspondence
            .native_witness_roles()
            .nth(self.compared().role)
            .expect("admitted output role")
            .0
    }
    pub fn output_field(
        &self,
    ) -> &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus {
        self.compared().field()
    }
    pub fn observed_output_revision(
        &self,
    ) -> worth_relational::facade::runtime::RelationalFieldRevision {
        self.compared().revision
    }
    pub fn decision(&self) -> WorthQueryProducerEvaluationDecision {
        use worth_signal::facade::diagnostics::EvaluationVerdict;
        match &self
            .native()
            .computed_evaluation()
            .expect("computed comparison")
            .verdict
        {
            EvaluationVerdict::Recomputed => WorthQueryProducerEvaluationDecision::Changed,
            EvaluationVerdict::Suppressed { .. } => {
                WorthQueryProducerEvaluationDecision::Equivalent
            }
            EvaluationVerdict::Deferred { .. } => {
                unreachable!("an exact computed pair is never deferred")
            }
        }
    }
    pub fn same_evaluation_as(&self, other: &Self) -> bool {
        self.compared()
            .comparison
            .same_comparison(&other.compared().comparison)
    }
    pub(in super::super) fn native(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeConditionalCausalObservation {
        self.compared().comparison.candidate()
    }
    pub(in super::super) fn baseline(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeConditionalCausalObservation {
        self.compared().comparison.baseline()
    }
    pub(in super::super) fn performed_basis(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchBasisDescriptor {
        &self.compared().performed_basis
    }
    pub(in super::super) fn field_meaning(&self) -> [u8; 32] {
        self.compared().field_meaning()
    }
}

pub(super) fn evaluate<Schema: worth_query_installation::facade::ApplicationSchema>(
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    execution: &ExecutionRequest<'_, '_>,
    target: &WorthQueryApplicationInspectionTarget,
    source: &WorthQueryApplicationCommitReceipt,
    diagnostic: &DiagnosticComparisonAdmission<'_>,
    source_diagnostic: Option<&DiagnosticComparisonAdmission<'_>>,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<WorthQueryApplicationProducerEvaluation, CaptureStop<ConnectionRefusal>> {
    evaluate_installed(
        InstalledEvaluationFields {
            layout: &runtime.primary_provider.graph.layout,
            contracts: runtime.installed_schema().native_contracts(),
        },
        execution,
        target,
        source,
        diagnostic,
        source_diagnostic,
        memory,
    )
}
struct InstalledEvaluationFields<'a> {
    layout: &'a crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contracts:
        &'a worth_query_installation::facade::WorthQueryInstalledApplicationSchemaContractCatalog,
}
fn evaluate_installed(
    fields: InstalledEvaluationFields<'_>,
    execution: &ExecutionRequest<'_, '_>,
    target: &WorthQueryApplicationInspectionTarget,
    source: &WorthQueryApplicationCommitReceipt,
    diagnostic: &DiagnosticComparisonAdmission<'_>,
    source_diagnostic: Option<&DiagnosticComparisonAdmission<'_>>,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<WorthQueryApplicationProducerEvaluation, CaptureStop<ConnectionRefusal>> {
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
        if !matches!(&original.source.selection, WorthQueryApplicationBasisSelectionIdentity::Product(read) if supplier.read() == read)
        {
            return Err(MapKernelFailure::Domain(Denial::ProducerUnavailable.into()));
        }
        let OriginalSettledProducerSource::Committed {
            changes,
            inputs,
            evaluation,
            ..
        } = supplier.source()
        else {
            return Err(refuse(WorthQueryProducerEvaluationDenial::NotPerformed));
        };
        let inputs = inputs.as_ref().ok_or(MapKernelFailure::Domain(
            Denial::PerformedInputsUnavailable.into(),
        ))?;
        let observed = overlap::ObservationConnection {
            original,
            diagnostic,
            layout: fields.layout,
            contracts: fields.contracts,
        };
        source::require_input(
            &observed,
            context,
            target,
            source.committed_changes(),
            inputs,
            source_diagnostic,
        )?;
        let evaluation = evaluation
            .as_ref()
            .ok_or_else(|| refuse(WorthQueryProducerEvaluationDenial::EvaluationUnavailable))?;
        // Availability and omission are also protected by this declared field's
        // actual query projection and diagnostic mask, before reading its state.
        let field = evaluation.field();
        let output = supplier
            .correspondence()
            .native_witness_roles()
            .find(|(role, _, entity, _)| *role == evaluation.role() && *entity == field.entity())
            .map(|(_, _, _, entity)| entity);
        let mut projected = false;
        if observed.projected(field.entity(), field.aspect(), field.field()) {
            for row in original.source.original_row_footprints() {
                for seen in &row.aspects {
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                        context, 1,
                    )
                    .map_err(MapKernelFailure::Stop)?;
                    projected |= row.complete
                        && Some(seen.entity) == output
                        && seen.entity_name == field.entity()
                        && seen.aspect.as_str() == field.aspect()
                        && seen.field.as_str() == field.field();
                }
            }
        }
        if !projected {
            return Err(refuse(
                WorthQueryProducerEvaluationDenial::ObservedFieldUnconnected,
            ));
        }
        let compared = match &evaluation.state {
            ProducerOutputEvaluationState::Compared(compared) => compared,
            ProducerOutputEvaluationState::CaptureUnavailable(_) => {
                return Err(refuse(
                    WorthQueryProducerEvaluationDenial::OriginalCaptureUnavailable,
                ))
            }
            ProducerOutputEvaluationState::Ineligible(cause) => {
                return Err(refuse(WorthQueryProducerEvaluationDenial::Ineligible(
                    *cause,
                )))
            }
        };
        if !observed_field(&observed, context, compared)? {
            return Err(refuse(
                WorthQueryProducerEvaluationDenial::ObservedFieldUnconnected,
            ));
        }
        Ok(WorthQueryApplicationProducerEvaluation {
            source: source.committed_changes().clone(),
            producer: changes.clone(),
            correspondence: supplier.retain_correspondence(),
            evaluation: Arc::clone(evaluation),
            _inputs: inputs.clone(),
            _disclosure: disclosure,
        })
    })
}
fn refuse(cause: WorthQueryProducerEvaluationDenial) -> MapKernelFailure<ConnectionRefusal> {
    MapKernelFailure::Domain(WorthQueryProducerConnectionDenial::Evaluation(cause).into())
}
fn observed_field(
    observed: &overlap::ObservationConnection<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    compared: &crate::domain_computation::primary_graph::application_contribution::ComparedProducerOutput,
) -> Result<bool, MapKernelFailure<ConnectionRefusal>> {
    let locus = compared.field();
    for row in observed.original.source.original_row_footprints() {
        for field in &row.aspects {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)
                .map_err(MapKernelFailure::Stop)?;
            if row.complete
                && field.entity == compared.entity
                && field.entity_name == locus.entity()
                && field.aspect.as_str() == locus.aspect()
                && field.field.as_str() == locus.field()
                && field.native_revision == Some(compared.revision)
                && field.contract_revision == compared.contract_revision()
                && observed.projected(locus.entity(), locus.aspect(), locus.field())
            {
                let coordinate = observed
                    .layout
                    .resolve_declared_field_coordinate(
                        observed.contracts,
                        locus.entity(),
                        &field.aspect,
                        &field.field,
                        |work| {
                            crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                                context, work,
                            )
                        },
                    )
                    .map_err(MapKernelFailure::Stop)?;
                if coordinate.is_some_and(|coordinate| {
                    coordinate.entity_kind() == compared.kind
                        && coordinate.contract_revision() == compared.contract_revision()
                }) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
