//! New meaning selects a new portable profile; old bodies retain their historical version.
use super::*;
pub(super) fn select(body: &ApplicationOperationBodyProgram) -> u16 {
    let scoped = body.explicit_scope_entity().is_some() || body.is_creation_only();
    if body.has_multiple_child_links() {39}
    else if body.creation().is_some_and(|c|!c.peers().is_empty()||!c.path_links().is_empty()) {38}
    else if !body.reference_inputs().is_empty() {37}
    else if body.creation().is_some_and(|creation| !creation.children().is_empty()) { 36 }
    else if body.creation().is_some_and(|creation| creation.reuses_current_if_unchanged()) { 35 }
    else if body.creation().is_some_and(|creation| creation.retires_predecessor()) { 34 }
    else if !body.path_value_assignments().is_empty() { 33 }
    else if !body.path_input_expectations().is_empty() { 32 }
    else if !body.retirements().is_empty() { 31 }
    else if body.scope_fields().iter().any(|step| matches!(step, ApplicationOperationBodyScopeField::ExpectInput { .. })) { 30 }
    else if body.creation().is_some_and(|c| c.subject_impact_bound().is_some() || c.fields().iter().any(|f| f.path_source().is_some())) { 29 }
    else if !body.values().is_empty() || body.external_emission().is_some_and(|emission| matches!(emission.source, worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::Evaluation { .. })) { 28 }
    else if body.external_emission().is_some_and(|emission| matches!(emission.source, worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::QueryContext { .. })) { 27 }
    else if body.external_emission().is_some() { 26 }
    else if body.requires_query_context() { 25 }
    else if body.requirements().iter().flat_map(|requirement| requirement.operands()).any(|operand|
        matches!(operand.source(), worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource::RelatedFieldValues { .. })) { 24 }
    else if body.succession().is_some_and(|s| s.output_match().is_some()) { 23 }
    else if body.outputs().iter().any(|output| matches!(output.target, worth_query_declaration::facade::application_schema::ApplicationOperationBodyOutputTarget::Related { .. })) { 22 }
    else if body.succession().is_some() { 21 }
    else if body.creation().is_some_and(|creation| creation.fields().iter().any(|step| step.related_source().is_some())) { 20 }
    else if body.creation().is_some_and(|creation| creation.links().iter().any(|link|
        link.posture == worth_query_declaration::facade::application_schema::ApplicationOperationBodyCreationLinkPosture::ReplaceSingle)) {
        19
    } else if body.requirements().iter().flat_map(|requirement| requirement.operands()).any(|operand|
        matches!(operand.source(), worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource::RelatedSignedEvidence { .. })) {
        18
    } else if body.creation().is_some_and(|creation| creation.enrollment().is_some()) {
        17
    } else if body.requirements().iter().flat_map(|requirement| requirement.operands()).any(|operand|
        matches!(operand.source(), worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource::SignedEvidence(_))) {
        16
    } else if body
        .creation()
        .is_some_and(|creation| creation.signing_lifecycle().is_some())
    {
        15
    } else if body.creation().is_some_and(|creation| {
        creation.fields().iter().any(|step| {
            matches!(
                step,
                ApplicationOperationBodyCreatedField::CapturedScopeField { .. }
            )
        })
    }) {
        14
    } else if signing::uses_signed_sources(body) {
        13
    } else if body.creation().is_some_and(|creation| {
        creation
            .fields()
            .iter()
            .any(|step| step.principal_source().is_some())
    }) {
        12
    } else if body
        .creation()
        .is_some_and(|creation| !creation.links().is_empty())
    {
        11
    } else if !body.outputs().is_empty() {
        10
    } else if body.has_optional_inputs() {
        9
    } else if scoped {
        8
    } else if !body.requirements().is_empty() {
        7
    } else if body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::RequirePresent { .. }
        )
    }) {
        6
    } else if body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                | ApplicationOperationBodyScopeField::ClearOptional { .. }
        )
    }) {
        5
    } else if body.creation().is_some_and(|creation| {
        creation
            .fields()
            .iter()
            .any(|step| step.source_field().is_some())
    }) {
        4
    } else if body.creation().is_some() {
        3
    } else if body
        .scope_fields()
        .iter()
        .any(|step| step.input_member().is_some())
    {
        2
    } else {
        1
    }
}
