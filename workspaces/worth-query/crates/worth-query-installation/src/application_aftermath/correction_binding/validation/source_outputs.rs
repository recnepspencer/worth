//! Historical field manifests resolve against the actual source and schema.
use worth_query_declaration::facade::{
    application_aftermath::DeclaredCorrectionOutputField,
    application_operation::{
        ApplicationMutationBindingDescriptor, ApplicationMutationOutputPosture,
        ApplicationMutationOutputRoleCardinality,
    },
    application_schema::ApplicationSchemaMember,
};

pub(super) fn validate(
    members: &[ApplicationSchemaMember],
    source: &ApplicationMutationBindingDescriptor,
    fields: &[DeclaredCorrectionOutputField],
) -> Result<(), ()> {
    for (index, field) in fields.iter().enumerate() {
        let parts = field.parts();
        if parts.source_binding != *source.description().binding_identity()
            || fields[..index].iter().any(|other| {
                let other = other.parts();
                other.role == parts.role
                    && other.entity == parts.entity
                    && other.aspect == parts.aspect
                    && other.field == parts.field
            })
        {
            return Err(());
        }
        let mut roles = source
            .output_roles()
            .iter()
            .filter(|role| role.name() == parts.role);
        let Some(role) = roles.next() else {
            return Err(());
        };
        if roles.next().is_some()
            || role.entity() != parts.entity
            || role.posture() != ApplicationMutationOutputPosture::Create
            || role.cardinality() != ApplicationMutationOutputRoleCardinality::ExactlyOne
        {
            return Err(());
        }
        let mut declared = members.iter().filter(|member| {
            matches!(member,
            ApplicationSchemaMember::Field { entity, aspect, field, .. }
            if entity == &parts.entity && aspect == &parts.aspect && field == &parts.field)
        });
        let Some(actual) = declared.next() else {
            return Err(());
        };
        if declared.next().is_some() || !field.matches_schema_field(actual) {
            return Err(());
        }
    }
    Ok(())
}
