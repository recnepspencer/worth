//! Selected canonical source effects retain their actual original envelope owners.
use super::super::{admission::InspectionAdmissionDenial, source_original::SourceOriginal};
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::DiagnosticComparisonAdmission,
    primary_graph::{
        admitted_capture::{checkpoint, run_retained_stage, CaptureStop},
        application_attempt::MaterializedCommittedField,
        WorthQueryPrimaryGraphLayout,
    },
};
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial, MapKernelFailure,
};
use worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchemaContractCatalog;
pub use worth_relational::facade::publication::PublishedAuthoritativeFieldEffect as WorthQueryMaterializedSourceFieldEffect;

pub struct WorthQueryMaterializedSourceField {
    slot: Arc<ApplicationQueryResultSlotKey>,
    path: Arc<str>,
    original: MaterializedCommittedField,
}
impl WorthQueryMaterializedSourceField {
    pub fn output_name(&self) -> Option<&str> {
        self.slot.output_name()
    }
    pub fn result_path(&self) -> &str {
        &self.path
    }
    pub fn effect(&self) -> WorthQueryMaterializedSourceFieldEffect {
        self.original.effect()
    }
    /// Original canonical written value only. Unchanged and clear effects supply no value.
    pub fn value(&self) -> Option<&WorthQueryMaterializedFieldValue> {
        self.original.value()
    }
}
pub struct WorthQueryMaterializedSourceEffects {
    original: SourceOriginal,
    fields: ExecutionArray<WorthQueryMaterializedSourceField>,
}
impl WorthQueryMaterializedSourceEffects {
    pub fn source_commit(&self) -> &worth_relational::facade::history::RelationalCommitReceipt {
        self.original.changes.commit_reference()
    }
    pub fn fields(&self) -> impl ExactSizeIterator<Item = &WorthQueryMaterializedSourceField> {
        self.fields.iter()
    }
    pub(in super::super) fn matches(&self, source: &SourceOriginal) -> bool {
        self.original.same_original(source)
    }
}
pub(in super::super) enum SourceDetail {
    NotRequested,
    Unselected,
    UnsupportedShape,
    Provided(WorthQueryMaterializedSourceEffects),
}
impl SourceDetail {
    pub(in super::super) fn outcome(&self) -> WorthQueryMaterializedFamilyOutcome {
        use WorthQueryMaterializedFamilyOutcome as Outcome;
        match self {
            Self::NotRequested => Outcome::NotRequested,
            Self::Unselected => Outcome::RequestedButNotProvidedBySelectedObservationMode,
            Self::UnsupportedShape => Outcome::UnsupportedSourceProjectionShape,
            Self::Provided(_) => Outcome::Provided,
        }
    }
    pub(in super::super) fn label(&self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::Unselected => "not_selected",
            Self::UnsupportedShape => "unsupported_source_projection_shape",
            Self::Provided(_) => "provided_original_canonical_source_effects",
        }
    }
}
pub(in super::super) enum SourceMaterializationDenial {
    UnresolvedField,
    MissingOriginalValue,
    Array(ExecutionRequestArrayDenial),
}
fn array_stop(cause: ExecutionRequestArrayDenial) -> InspectionAdmissionDenial {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => {
            InspectionAdmissionDenial::RichSource(CaptureStop::Request(cause))
        }
        cause => InspectionAdmissionDenial::RichSource(CaptureStop::Kernel(
            worth_execution::ScopeStop::Failure(MapKernelFailure::Domain(
                SourceMaterializationDenial::Array(cause),
            )),
        )),
    }
}
/// Both the selected original source and the fresh scoped Diagnostic loan are required.
pub(in super::super) fn capture(
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    source: &SourceOriginal,
    source_kind: worth_relational::facade::identity::KindId,
    admission: DiagnosticComparisonAdmission<'_>,
    layout: &WorthQueryPrimaryGraphLayout,
    contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
) -> Result<SourceDetail, InspectionAdmissionDenial> {
    let graph = admission.graph();
    let supported = run_retained_stage(execution, memory, |context, _| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        if layout.entity_kind(graph.root_entity()) != Some(source_kind)
            || !graph.relations().is_empty()
        {
            return Ok(false);
        }
        for projection in graph.projections() {
            let work = projection
                .parent_path()
                .len()
                .checked_add(projection.entity().len())
                .and_then(|n| n.checked_add(graph.root_entity().len()))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(MapKernelFailure::Stop(
                    worth_execution::MapKernelStop::WorkCounterOverflow,
                ))?;
            checkpoint(context, work).map_err(MapKernelFailure::Stop)?;
            if projection.parent_path() != "root" || projection.entity() != graph.root_entity() {
                return Ok(false);
            }
        }
        Ok(true)
    })
    .map_err(InspectionAdmissionDenial::RichSource)?;
    if !supported {
        return Ok(SourceDetail::UnsupportedShape);
    }
    let mut fields = ExecutionRequestArrayBuilder::allocate(*execution, graph.projections().len())
        .map_err(array_stop)?;
    for projection in graph.projections() {
        let field = run_retained_stage(execution, memory, |context, _| {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            let coordinate = layout
                .resolve_declared_field_coordinate(
                    contracts,
                    projection.entity(),
                    projection.aspect_key(),
                    projection.field_key(),
                    |work| checkpoint(context, work),
                )
                .map_err(MapKernelFailure::Stop)?
                .ok_or(MapKernelFailure::Domain(
                    SourceMaterializationDenial::UnresolvedField,
                ))?;
            let original = source
                .changes
                .materialize_field(context, source.disclosure.scope().subject(), coordinate)
                .map_err(MapKernelFailure::Stop)?;
            if original.effect() == WorthQueryMaterializedSourceFieldEffect::Present
                && original.value().is_none()
            {
                return Err(MapKernelFailure::Domain(
                    SourceMaterializationDenial::MissingOriginalValue,
                ));
            }
            Ok(WorthQueryMaterializedSourceField {
                slot: projection.slot_key_identity(),
                path: projection.result_path_identity(),
                original,
            })
        })
        .map_err(InspectionAdmissionDenial::RichSource)?;
        fields.push(field).map_err(array_stop)?;
    }
    Ok(SourceDetail::Provided(
        WorthQueryMaterializedSourceEffects {
            original: source.clone(),
            fields: fields.seal().map_err(array_stop)?,
        },
    ))
}
