//! Member tag 59: a capability whose standing tenure, extent liveness, UTC or
//! local-date validity, or self-grant rule tags 7 and 33 cannot express.
//!
//! A contract tags 7 or 33 can express is always written under them, byte for
//! byte as before; decoding such a contract from tag 59 is refused, so every
//! contract has exactly one encoding and older readers refuse tag 59 outright.
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityConstraintDefinition, ApplicationCapabilityCurrentnessDefinition,
    ApplicationCapabilityDelegationDefinition, ApplicationCapabilityDelegationLineage,
    ApplicationCapabilityGrantTenure, ApplicationCapabilitySelfGrantRule,
    ApplicationCapabilityTargetDefinition, ApplicationCapabilityValidityDefinition,
    ApplicationCapabilityValidityTimeline, ApplicationCapabilityWorkflowDefinition,
    ErasedApplicationCapabilityContract,
};

use super::super::super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::super::super::sequence::{decode_sequence, write_sequence};
use super::super::super::super::wire_vocabulary::{decode_type_identity, write_type_identity};
use super::{bindings, resource_path, scope};
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

/// Whether tags 7 and 33 cannot express `contract`.
pub(super) fn requires_extended_record(contract: &ErasedApplicationCapabilityContract) -> bool {
    let currentness = contract.constraints().currentness();
    let delegation = contract.delegation();
    currentness.workflow().is_none()
        || delegation.lineage().is_none()
        || !currentness.extent_status().is_empty()
        || !currentness.validity().timeline().counts_unix_epoch_units()
        || !currentness.validity().is_closed()
        || delegation.self_grant() != ApplicationCapabilitySelfGrantRule::Permitted
}

pub(super) fn write_target(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityTargetDefinition,
) -> Result<(), Denial> {
    bindings::write_value(output, value.action())?;
    bindings::write_relation(output, value.resource())?;
    if value.resource_path().is_direct() {
        output.u16(0)?;
    } else {
        output.u16(1)?;
        resource_path::write(output, value.resource_path())?;
    }
    scope::write_relation_dimension(output, value.relation())?;
    scope::write_field_dimension(output, value.field())?;
    bindings::write_value(output, value.purpose())
}

pub(super) fn decode_target(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityTargetDefinition, Denial> {
    let action = bindings::decode_value(input)?;
    let head = bindings::decode_relation(input)?;
    let resource = match input.u16()? {
        0 => head.into(),
        1 => resource_path::decode(input, budget, head)?,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(ApplicationCapabilityTargetDefinition::new(
        action,
        resource,
        scope::decode_relation_dimension(input)?,
        scope::decode_field_dimension(input)?,
        bindings::decode_value(input)?,
    ))
}

pub(super) fn write_constraints(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityConstraintDefinition,
) -> Result<(), Denial> {
    scope::write_field_dimension(output, value.magnitude())?;
    scope::write_cardinality(output, value.cardinality())?;
    write_currentness(output, value.currentness())?;
    output.text(value.context())?;
    write_type_identity(output, &value.context_identity())
}

pub(super) fn decode_constraints(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityConstraintDefinition, Denial> {
    use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
    use worth_query_declaration::facade::application_capability::WorthQueryPortableApplicationCapabilityConstraintParts as Parts;
    Ok(
        ApplicationCapabilityConstraintDefinition::from_untrusted_parts(
            ARCHIVE_DECODE,
            Parts {
                magnitude: scope::decode_field_dimension(input)?,
                cardinality: scope::decode_cardinality(input)?,
                currentness: decode_currentness(input, budget)?,
                context: input.text()?.to_owned(),
                context_type: decode_type_identity(input)?,
            },
        ),
    )
}

fn write_currentness(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityCurrentnessDefinition,
) -> Result<(), Denial> {
    bindings::write_value(output, value.active_status())?;
    match value.tenure() {
        ApplicationCapabilityGrantTenure::Workflow(workflow) => {
            output.u16(1)?;
            bindings::write_field(output, workflow.grant())?;
            bindings::write_field(output, workflow.resource())?;
        }
        ApplicationCapabilityGrantTenure::Standing => output.u16(2)?,
    }
    write_validity(output, value.validity())?;
    write_sequence(output, value.extent_status(), bindings::write_value)
}

fn decode_currentness(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityCurrentnessDefinition, Denial> {
    let active = bindings::decode_value(input)?;
    let workflow = match input.u16()? {
        1 => Some(ApplicationCapabilityWorkflowDefinition::new(
            bindings::decode_field(input)?,
            bindings::decode_field(input)?,
        )),
        2 => None,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let validity = decode_validity(input)?;
    // An open window is a standing grant's; refuse it on a workflow grant at
    // the raw decode, before installation would.
    if workflow.is_some() && !validity.is_closed() {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let extent = decode_sequence(input, budget, 8, |input, _| bindings::decode_value(input))?;
    if extent.len() > ApplicationCapabilityCurrentnessDefinition::MAXIMUM_EXTENT_STATUS {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let currentness = match workflow {
        Some(workflow) => {
            ApplicationCapabilityCurrentnessDefinition::new(active, workflow, validity)
        }
        None => ApplicationCapabilityCurrentnessDefinition::standing(active, validity),
    };
    Ok(extent.into_iter().fold(
        currentness,
        ApplicationCapabilityCurrentnessDefinition::require_active,
    ))
}

/// An open lower bound sets this bit of the timeline word.
const OPEN_BELOW: u16 = 1 << 8;
/// An open upper bound sets this bit of the timeline word.
const OPEN_ABOVE: u16 = 1 << 9;

/// Grant validity only; elevation validity stays on the tag 7/33 timelines.
/// A closed window writes its timeline code and both fields exactly as
/// before; an open bound sets its bit and writes no field.
fn write_validity(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityValidityDefinition,
) -> Result<(), Denial> {
    let timeline = match value.timeline() {
        ApplicationCapabilityValidityTimeline::UnixEpochSeconds => 1,
        ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds => 2,
        ApplicationCapabilityValidityTimeline::UtcTimestamp => 3,
        ApplicationCapabilityValidityTimeline::LocalDate => 4,
    };
    let open = |bound: Option<_>, bit| if bound.is_none() { bit } else { 0 };
    output.u16(
        timeline | open(value.not_before(), OPEN_BELOW) | open(value.not_after(), OPEN_ABOVE),
    )?;
    for bound in value.bounds() {
        bindings::write_field(output, bound)?;
    }
    Ok(())
}

fn decode_validity(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityValidityDefinition, Denial> {
    let word = input.u16()?;
    let timeline = match word & !(OPEN_BELOW | OPEN_ABOVE) {
        1 => ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
        2 => ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds,
        3 => ApplicationCapabilityValidityTimeline::UtcTimestamp,
        4 => ApplicationCapabilityValidityTimeline::LocalDate,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let mut bound = |open: u16| {
        (word & open == 0)
            .then(|| bindings::decode_field(input))
            .transpose()
    };
    let not_before = bound(OPEN_BELOW)?;
    let not_after = bound(OPEN_ABOVE)?;
    // Only the shapes the declaration constructors produce decode.
    ApplicationCapabilityValidityDefinition::from_portable_parts(timeline, not_before, not_after)
        .ok_or_else(|| Denial::new(Kind::UnsupportedRecordVariant))
}

pub(super) fn write_delegation(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityDelegationDefinition,
) -> Result<(), Denial> {
    match value.lineage() {
        Some(lineage) => {
            output.u16(1)?;
            bindings::write_relation(output, lineage.parent())?;
            bindings::write_relation(output, lineage.grantor())?;
            bindings::write_field(output, lineage.limit())?;
        }
        None => output.u16(2)?,
    }
    bindings::write_relation(output, value.grantee())?;
    output.u16(match value.self_grant() {
        ApplicationCapabilitySelfGrantRule::Permitted => 1,
        ApplicationCapabilitySelfGrantRule::Forbidden => 2,
    })?;
    scope::write_provenance_and_programs(output, value)
}

pub(super) fn decode_delegation(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityDelegationDefinition, Denial> {
    let lineage = match input.u16()? {
        1 => Some(ApplicationCapabilityDelegationLineage::new(
            bindings::decode_relation(input)?,
            bindings::decode_relation(input)?,
            bindings::decode_field(input)?,
        )),
        2 => None,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let grantee = bindings::decode_relation(input)?;
    let self_grant = match input.u16()? {
        1 => ApplicationCapabilitySelfGrantRule::Permitted,
        2 => ApplicationCapabilitySelfGrantRule::Forbidden,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    scope::decode_provenance_and_programs(input, budget, lineage, grantee, self_grant)
}
