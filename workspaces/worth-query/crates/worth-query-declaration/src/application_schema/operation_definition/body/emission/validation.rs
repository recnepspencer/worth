use crate::application_schema::{
    ApplicationOperationBodyProgram, ApplicationOperationProgramTarget,
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};

pub(crate) fn validate_emission(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[Member],
) -> Result<(), Denial> {
    let Some(emission) = body.input_emission() else {
        return Ok(());
    };
    let bound = emission.maximum_payload_bytes != 0
        && members.iter().any(|member| matches!(member, Member::Effect { effect, payload_type } if effect == &emission.effect && payload_type == &emission.payload_binding))
        && members.iter().any(|member| matches!(member, Member::Operation { operation: name, input_type } if name == operation && input_type == &emission.input_binding))
        && members.iter().any(|member| matches!(member, Member::OperationExternalEffect { operation: name, effect, rust_payload_type, protocol, maximum_payload_bytes, .. }
            if name == operation && effect == &emission.effect && rust_payload_type == &emission.payload_binding && protocol == &emission.protocol && maximum_payload_bytes == &emission.maximum_payload_bytes))
        && members.iter().any(|member| matches!(member, Member::OperationProgram { operation: name, target: ApplicationOperationProgramTarget::Emit { effect } }
            if name == operation && effect == &emission.effect));
    if bound {
        Ok(())
    } else {
        Err(Denial::MissingOperationProgramDependency)
    }
}
