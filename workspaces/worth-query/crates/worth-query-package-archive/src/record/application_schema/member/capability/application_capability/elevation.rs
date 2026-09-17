use std::time::Duration;

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityElevationStates, ApplicationCapabilityMandatoryReviewDefinition,
    WorthQueryPortableApplicationCapabilityElevationDefinitionParts,
    WorthQueryPortableApplicationCapabilityElevationLifecycleParts,
    WorthQueryPortableApplicationCapabilityElevationRuleParts,
    WorthQueryPortableApplicationCapabilityLifecycleEffectParts,
    WorthQueryPortableApplicationCapabilityTransitionBindingParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::super::wire_vocabulary::{decode_type_identity, write_type_identity};
use super::{bindings, scope};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryPortableApplicationCapabilityElevationRuleParts,
) -> Result<(), Denial> {
    match value {
        WorthQueryPortableApplicationCapabilityElevationRuleParts::NotApplicable => output.u16(0),
        WorthQueryPortableApplicationCapabilityElevationRuleParts::Governed(value) => {
            output.u16(2)?;
            write_definition(output, value)
        }
    }
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortableApplicationCapabilityElevationRuleParts, Denial> {
    match input.u16()? {
        0 => Ok(WorthQueryPortableApplicationCapabilityElevationRuleParts::NotApplicable),
        2 => decode_definition(input, budget)
            .map(WorthQueryPortableApplicationCapabilityElevationRuleParts::Governed),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

fn write_definition(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryPortableApplicationCapabilityElevationDefinitionParts,
) -> Result<(), Denial> {
    bindings::write_field(output, &value.identity)?;
    bindings::write_field(output, &value.reason)?;
    bindings::write_field(output, &value.status)?;
    bindings::write_field(output, &value.closed_at)?;
    for state in value.states.values() {
        bindings::write_value(output, state)?;
    }
    scope::write_validity(output, &value.validity)?;
    output.u64(value.maximum_duration.as_secs())?;
    output.u32(value.maximum_duration.subsec_nanos())?;
    bindings::write_relation(output, &value.requester)?;
    bindings::write_relation(output, &value.approver)?;
    bindings::write_relation(output, &value.grant)?;
    match value.resource_relation.as_ref() {
        None => output.u16(0)?,
        Some(value) => {
            output.u16(1)?;
            bindings::write_relation(output, value)?;
        }
    }
    write_lifecycle(output, &value.lifecycle)?;
    write_review(output, &value.review)
}

fn decode_definition(
    input: &mut BinaryInput<'_>,
    _budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortableApplicationCapabilityElevationDefinitionParts, Denial> {
    let identity = bindings::decode_field(input)?;
    let reason = bindings::decode_field(input)?;
    let status = bindings::decode_field(input)?;
    let closed_at = bindings::decode_field(input)?;
    let states = ApplicationCapabilityElevationStates::new(
        bindings::decode_value(input)?,
        bindings::decode_value(input)?,
        bindings::decode_value(input)?,
        bindings::decode_value(input)?,
    );
    let validity = scope::decode_validity(input)?;
    let seconds = input.u64()?;
    let nanos = input.u32()?;
    if nanos >= 1_000_000_000 {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    let maximum_duration = Duration::new(seconds, nanos);
    let requester = bindings::decode_relation(input)?;
    let approver = bindings::decode_relation(input)?;
    let grant = bindings::decode_relation(input)?;
    let resource_relation = match input.u16()? {
        0 => None,
        1 => Some(bindings::decode_relation(input)?),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let lifecycle = decode_lifecycle(input)?;
    let review = decode_review(input)?;
    Ok(
        WorthQueryPortableApplicationCapabilityElevationDefinitionParts {
            identity,
            reason,
            status,
            closed_at,
            states,
            validity,
            maximum_duration,
            requester,
            approver,
            grant,
            resource_relation,
            lifecycle,
            review,
        },
    )
}

fn write_lifecycle(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryPortableApplicationCapabilityElevationLifecycleParts,
) -> Result<(), Denial> {
    bindings::write_context_slot(output, &value.elevation_slot)?;
    bindings::write_context_slot(output, &value.review_slot)?;
    for transition in [
        &value.request,
        &value.approve,
        &value.revoke,
        &value.complete_review,
    ] {
        write_transition(output, transition)?;
    }
    Ok(())
}
fn decode_lifecycle(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryPortableApplicationCapabilityElevationLifecycleParts, Denial> {
    Ok(
        WorthQueryPortableApplicationCapabilityElevationLifecycleParts {
            elevation_slot: bindings::decode_context_slot(input)?,
            review_slot: bindings::decode_context_slot(input)?,
            request: decode_transition(input)?,
            approve: decode_transition(input)?,
            revoke: decode_transition(input)?,
            complete_review: decode_transition(input)?,
        },
    )
}

fn write_transition(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryPortableApplicationCapabilityTransitionBindingParts,
) -> Result<(), Denial> {
    output.text(&value.capability)?;
    write_type_identity(output, &value.capability_type)?;
    bindings::write_operation(output, &value.operation)?;
    match value.lifecycle_effect.as_ref() {
        None => output.u16(0),
        Some(value) => {
            output.u16(1)?;
            output.text(&value.effect)?;
            output.text(&value.effect_type)?;
            write_type_identity(output, &value.payload_type)
        }
    }
}
fn decode_transition(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryPortableApplicationCapabilityTransitionBindingParts, Denial> {
    let capability = input.text()?.to_owned();
    let capability_type = decode_type_identity(input)?;
    let operation = bindings::decode_operation(input)?;
    let lifecycle_effect = match input.u16()? {
        0 => None,
        1 => Some(
            WorthQueryPortableApplicationCapabilityLifecycleEffectParts {
                effect: input.text()?.to_owned(),
                effect_type: input.text()?.to_owned(),
                payload_type: decode_type_identity(input)?,
            },
        ),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(
        WorthQueryPortableApplicationCapabilityTransitionBindingParts {
            capability,
            capability_type,
            operation,
            lifecycle_effect,
        },
    )
}

fn write_review(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityMandatoryReviewDefinition,
) -> Result<(), Denial> {
    bindings::write_relation(output, value.relation())?;
    bindings::write_field(output, value.identity())?;
    bindings::write_value(output, value.kind())?;
    bindings::write_relation(output, value.scope())?;
    bindings::write_relation(output, value.reviewer())?;
    bindings::write_field(output, value.status())?;
    bindings::write_field(output, value.reviewed_at())?;
    bindings::write_value(output, value.required())?;
    bindings::write_value(output, value.completed())
}
fn decode_review(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityMandatoryReviewDefinition, Denial> {
    Ok(ApplicationCapabilityMandatoryReviewDefinition::new(
        bindings::decode_relation(input)?,
        bindings::decode_field(input)?,
        bindings::decode_value(input)?,
        bindings::decode_relation(input)?,
        bindings::decode_relation(input)?,
        bindings::decode_field(input)?,
        bindings::decode_field(input)?,
        bindings::decode_value(input)?,
        bindings::decode_value(input)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previous_governed_elevation_layout_fails_closed() {
        let legacy_tag = 1_u16.to_be_bytes();
        let mut input = BinaryInput::new(&legacy_tag);
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            legacy_tag.len() as u64,
            crate::limits::WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert_eq!(
            decode(&mut input, &mut budget).unwrap_err().kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}
