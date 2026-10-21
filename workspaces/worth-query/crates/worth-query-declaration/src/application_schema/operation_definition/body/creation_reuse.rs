//! Complete-content reuse is native selection meaning, not a command-wide no-op.
use super::{
    ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyCreationBuilder,
    ApplicationOperationBodyOutputTarget as Output, ApplicationOperationBodyProgram as Body,
};
use crate::application_schema::{
    ApplicationOperationDecisionReadTarget as Read, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};

impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    /// Reuse the current exclusive disposable selection when every declared field
    /// equals its validated initializer, including optional absence. Other effects run.
    /// Finish/readmission refuse signed creation, partial content and created outputs.
    pub fn reuse_current_if_unchanged(mut self) -> Self {
        self.reuse_current_if_unchanged = true;
        self
    }
}
impl Body {
    /// Structural profile only; complete field/read closure is checked at schema admission.
    pub fn has_supported_creation_reuse(&self) -> bool {
        let Some(creation) = self.creation().filter(|c| c.reuses_current_if_unchanged()) else {
            return true;
        };
        creation.path_links().is_empty()
            && creation.retires_predecessor()
            && self.has_supported_retiring_replacement()
            && creation.enrollment().is_none()
            && creation.signing_lifecycle().is_none()
            && creation.subject_impact_bound().is_none()
            && self.succession().is_none()
            && !self
                .outputs()
                .iter()
                .any(|output| matches!(output.target, Output::Creation { .. }))
            && creation.fields().iter().all(|field| {
                matches!(
                    field,
                    Field::Literal { .. } | Field::Input { .. } | Field::OptionalInput { .. }
                )
            })
    }
}

pub(super) fn validate(operation: &str, body: &Body, members: &[Member]) -> Result<(), Denial> {
    if !body.has_supported_creation_reuse() {
        return Err(Denial::program_body_dependency(operation));
    }
    let Some(creation) = body.creation().filter(|c| c.reuses_current_if_unchanged()) else {
        return Ok(());
    };
    for field in creation.fields() {
        let target = comparison_read(field);
        if !members.iter().any(|member| {
            matches!(member,
            Member::OperationDecisionRead { operation: declared, target: read }
                if declared == operation && read == &target)
        }) {
            return Err(Denial::decision_read_body_dependency(operation));
        }
    }
    // Include optional fields: silently ignoring an extra declared value would lie
    // about content equivalence. Existing admission rejects duplicate initializers.
    if members.iter().any(|member| matches!(member,
        Member::Field { entity, aspect, field, .. } if entity == creation.entity()
            && !creation.fields().iter().any(|step| step.field().aspect() == aspect && step.field().field() == field))) {
        return Err(Denial::program_body_dependency(operation));
    }
    Ok(())
}
pub(super) fn comparison_read(field: &Field) -> Read {
    Read::Field {
        entity: field.field().entity().to_owned(),
        aspect: field.field().aspect().to_owned(),
        field: field.field().field().to_owned(),
    }
}
