use super::super::{
    query_binding_reference,
    wire_vocabulary::{decode_type_identity, write_type_identity},
};
use super::resource_value;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::{application_query, decode_budget::RecordDecodeAttempt};
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};
use worth_query_declaration::facade::application_resource::{
    ApplicationResourceAction, ApplicationResourceAuthorization, ApplicationResourceLimits,
    ApplicationResourceRequestResponseProfile, ErasedApplicationResourceDefinition,
    WorthQueryPortableApplicationResourceParts,
};

pub(super) fn require_encoding_contract(
    definition: &ErasedApplicationResourceDefinition,
) -> Result<(), Denial> {
    definition
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationResourceDefinition,
) -> Result<(), Denial> {
    let parts = definition.parts();
    output.text(&parts.name)?;
    write_type_identity(output, &parts.resource_type)?;
    write_type_identity(output, &parts.provider_role)?;
    query_binding_reference::write(output, &parts.access)?;
    for value in [&parts.input, &parts.result, &parts.failure] {
        resource_value::write(output, value)?;
    }
    output.u32(parts.profile.retry_max_attempts)?;
    output.u64(parts.profile.retry_delay_milliseconds)?;
    output.u64(parts.profile.timeout_milliseconds)?;
    output.u32(parts.limits.maximum_live)?;
    output.u32(parts.limits.maximum_retained_results)?;
    output.u32(parts.limits.ingress_slots_per_request)?;
    output.u64(parts.limits.maximum_input_bytes)?;
    output.u64(parts.limits.maximum_frame_bytes)?;
    for action in ApplicationResourceAction::ALL {
        application_query::controls::write_authorization(
            output,
            parts.authorization.requirement(action),
        )?;
    }
    application_query::disclosure::write(output, &parts.disclosure)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ErasedApplicationResourceDefinition, Denial> {
    let name = input.text()?.to_owned();
    let resource_type = decode_type_identity(input)?;
    let provider_role = decode_type_identity(input)?;
    let access = query_binding_reference::decode(input)?;
    let input_binding = resource_value::decode(input)?;
    let result = resource_value::decode(input)?;
    let failure = resource_value::decode(input)?;
    let profile = ApplicationResourceRequestResponseProfile {
        retry_max_attempts: input.u32()?,
        retry_delay_milliseconds: input.u64()?,
        timeout_milliseconds: input.u64()?,
    };
    let limits = ApplicationResourceLimits {
        maximum_live: input.u32()?,
        maximum_retained_results: input.u32()?,
        ingress_slots_per_request: input.u32()?,
        maximum_input_bytes: input.u64()?,
        maximum_frame_bytes: input.u64()?,
    };
    let authorization = ApplicationResourceAuthorization::from_untrusted_requirements([
        application_query::controls::decode_authorization(input)?,
        application_query::controls::decode_authorization(input)?,
        application_query::controls::decode_authorization(input)?,
        application_query::controls::decode_authorization(input)?,
    ]);
    let disclosure = application_query::disclosure::decode(input, budget)?;
    let definition = ErasedApplicationResourceDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationResourceParts {
            name,
            resource_type,
            provider_role,
            access,
            input: input_binding,
            result,
            failure,
            profile,
            limits,
            authorization,
            disclosure,
        },
    );
    definition
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    Ok(definition)
}
