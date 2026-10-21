//! Cold descriptions must close every path, field codec and declared write.
use crate::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyProgram as Body,
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};
pub(in crate::application_schema::operation_definition::body) fn validate(
    operation: &str,
    body: &Body,
    members: &[Member],
) -> Result<(), Denial> {
    if body.has_duplicate_path_destinations() {
        return Err(Denial::DuplicateMember);
    }
    for step in body.path_value_assignments() {
        if !step.fits_body(body) || !step.target.has_closed_reads(operation, members) {
            return Err(Denial::decision_read_body_dependency(operation));
        }
        if !members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field, value_type, scalar_family: worth_foundational::facade::ScalarAspectType::UInt64, presence: ApplicationFieldPresence::Required, unit: None, frame: None, writable: true, .. }
            if entity == step.target.field.entity() && aspect == step.target.field.aspect() && field == step.target.field.field() && value_type == step.binding.as_str()))
            || !members.iter().any(|member| matches!(member, Member::OperationProgram { operation: declared, target } if declared == operation && target == &step.write_target())) {
            return Err(Denial::program_body_dependency(operation));
        }
    }
    Ok(())
}
