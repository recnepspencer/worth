use super::super::super::{
    decode_budget::RecordDecodeAttempt,
    sequence::{decode_sequence, write_sequence},
};
use super::super::wire_vocabulary::{decode_type_identity, write_type_identity};
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationDescription, ApplicationMutationDescriptionParts,
        ApplicationMutationOutputPosture as Posture, ApplicationMutationOutputRoleDescription,
        ApplicationMutationScopeDescription, ApplicationMutationScopeResolutionMode as Resolution,
    },
    application_schema::ApplicationSchemaMember,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    description: &ApplicationMutationDescription,
) -> Result<(), Denial> {
    write_type_identity(output, description.binding_identity())?;
    output.text(description.operation())?;
    write_type_identity(output, description.input_identity())?;
    write_type_identity(output, description.result_identity())?;
    write_type_identity(output, description.denial_identity())?;
    let scope = description.scope();
    output.text(&scope.entity)?;
    output.text(&scope.aspect)?;
    output.text(&scope.field)?;
    output.u16(match scope.resolution {
        Resolution::InputField => 1,
        Resolution::PrincipalIdentity => 2,
    })?;
    write_sequence(output, description.output_roles(), |output, role| {
        output.text(&role.name)?;
        output.text(&role.entity)?;
        output.u16(match role.posture {
            Posture::Preserve => 1,
            Posture::Create => 2,
            Posture::Retire => 3,
        })
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    budget.require_nesting_depth(2)?;
    let binding_identity = decode_type_identity(input)?;
    let operation = input.text()?.to_owned();
    let input_identity = decode_type_identity(input)?;
    let result_identity = decode_type_identity(input)?;
    let denial_identity = decode_type_identity(input)?;
    let scope = ApplicationMutationScopeDescription {
        entity: input.text()?.to_owned(),
        aspect: input.text()?.to_owned(),
        field: input.text()?.to_owned(),
        resolution: match input.u16()? {
            1 => Resolution::InputField,
            2 => Resolution::PrincipalIdentity,
            _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
        },
    };
    let output_roles = decode_sequence(input, budget, 10, |input, _| {
        Ok(ApplicationMutationOutputRoleDescription {
            name: input.text()?.to_owned(),
            entity: input.text()?.to_owned(),
            posture: match input.u16()? {
                1 => Posture::Preserve,
                2 => Posture::Create,
                3 => Posture::Retire,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            },
        })
    })?;
    Ok(ApplicationSchemaMember::ApplicationMutation {
        description: ApplicationMutationDescription::from_untrusted_parts(
            ApplicationMutationDescriptionParts {
                binding_identity,
                operation,
                input_identity,
                result_identity,
                denial_identity,
                scope,
                output_roles,
            },
        ),
    })
}
