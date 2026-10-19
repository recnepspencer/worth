//! New body meaning selects a profile without rewriting historical identities.
use crate::application_schema::{
    ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField,
};
pub(super) fn name(body: &ApplicationOperationBodyProgram) -> &'static str {
    if body.has_multiple_child_links() { "operation-body-v29" }
    else if body.creation().is_some_and(|c| !c.peers().is_empty() || !c.path_links().is_empty()) { "operation-body-v28" }
    else if !body.reference_inputs().is_empty() { "operation-body-v27" }
    else if body.creation().is_some_and(|creation| !creation.children().is_empty()) { "operation-body-v26" }
    else if body.creation().is_some_and(|creation| creation.reuses_current_if_unchanged()) { "operation-body-v25" }
    else if body.creation().is_some_and(|creation| creation.retires_predecessor()) { "operation-body-v24" }
    else if !body.path_value_assignments().is_empty() { "operation-body-v23" }
    else if !body.path_input_expectations().is_empty() { "operation-body-v22" }
    else if !body.retirements().is_empty() { "operation-body-v21" }
    else if body.scope_fields().iter().any(|step| matches!(step, ApplicationOperationBodyScopeField::ExpectInput { .. })) { "operation-body-v20" }
    else if body.creation().is_some_and(|c| c.subject_impact_bound().is_some() || c.fields().iter().any(|f| f.path_source().is_some())) { "operation-body-v19" }
    else if !body.values().is_empty() || body.external_emission().is_some_and(|emission| matches!(emission.source, crate::application_schema::ApplicationOperationBodyExternalPayloadSource::Evaluation { .. })) { "operation-body-v18" }
    else if body.external_emission().is_some_and(|emission| matches!(emission.source, crate::application_schema::ApplicationOperationBodyExternalPayloadSource::QueryContext { .. })) { "operation-body-v17" }
    else if body.external_emission().is_some() { "operation-body-v16" }
    else if body.succession().is_some_and(|s| s.output_match().is_some()) { "operation-body-v15" }
    else if body.outputs().iter().any(|output| matches!(output.target, crate::application_schema::ApplicationOperationBodyOutputTarget::Related { .. })) {
        "operation-body-v14"
    } else if body.succession().is_some() {
        "operation-body-v13"
    } else if body.creation().is_some_and(|creation| creation.fields().iter().any(|step| step.related_source().is_some())) {
        "operation-body-v12"
    } else if body.creation().is_some_and(|creation| creation.links().iter().any(|link|
        link.posture == crate::application_schema::ApplicationOperationBodyCreationLinkPosture::ReplaceSingle)) {
        "operation-body-v11"
    } else if body
        .creation()
        .is_some_and(|creation| creation.enrollment().is_some())
    {
        "operation-body-v10"
    } else if body
        .creation()
        .is_some_and(|creation| !creation.links().is_empty())
    {
        "operation-body-v9"
    } else if !body.outputs().is_empty() {
        "operation-body-v8"
    } else if body.has_optional_inputs() {
        "operation-body-v7"
    } else if body.explicit_scope_entity().is_some() || body.is_creation_only() {
        "operation-body-v6"
    } else if !body.requirements().is_empty() {
        "operation-body-v5"
    } else if body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::RequirePresent { .. }
        )
    }) {
        "operation-body-v4"
    } else if body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                | ApplicationOperationBodyScopeField::ClearOptional { .. }
        )
    }) {
        "operation-body-v3"
    } else if body.creation().is_some() {
        "operation-body-v2"
    } else {
        "operation-body-v1"
    }
}
