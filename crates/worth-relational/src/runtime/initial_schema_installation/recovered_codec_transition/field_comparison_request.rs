//! Translate native field comparison into the schema-installation contract.
use crate::schema::codec_field_comparison::{
    codec_field_changes_in_context, CodecFieldComparisonStop, OwnedCodecFieldChanges,
};
use crate::schema::data::RelationalSchemaRegistry;
use worth_relational_durability::internal_port::durability::transition_admission_denial::RelationalSchemaTransitionAdmissionDenialOwnerPort as _;

/// This comparison grants no retirement permission. Actual present records
/// separately require complete native producer-publication provenance.
pub(crate) fn codec_field_changes(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &RelationalSchemaRegistry,
    target: &RelationalSchemaRegistry,
) -> Result<OwnedCodecFieldChanges, crate::runtime::RelationalSchemaTransitionAdmissionDenial> {
    let compared=execution.in_scope(|lease| {
        worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
            .run_with_checkpoint(lease,None,|context| {
                codec_field_changes_in_context(source, target, context, lease)
            })
    }).map_err(|stop| crate::runtime::RelationalSchemaTransitionAdmissionDenial::request(stop.into()))
        .and_then(|result| result.map_err(|stop| crate::runtime::RelationalSchemaTransitionAdmissionDenial::request(stop.into())))
        .and_then(|result| result.0.map_err(|stop| match stop {
            CodecFieldComparisonStop::Request(cause) => crate::runtime::RelationalSchemaTransitionAdmissionDenial::request(cause),
            CodecFieldComparisonStop::Contract(detail) => crate::runtime::RelationalSchemaTransitionAdmissionDenial::new(crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::TargetContractMismatch,detail),
        }));
    crate::execution::check_request(execution)
        .map_err(crate::runtime::RelationalSchemaTransitionAdmissionDenial::request)?;
    compared
}
