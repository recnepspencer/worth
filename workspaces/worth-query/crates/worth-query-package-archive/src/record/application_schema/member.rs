use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::decode_budget::RecordDecodeAttempt;

mod authorization;
mod capability;
mod operation;
mod schema;
mod vocabulary;

pub(super) fn require_nesting_depth(
    member: &ApplicationSchemaMember,
    maximum_depth: u32,
) -> Result<(), Denial> {
    capability::require_nesting_depth(member, maximum_depth)
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    output.u16(member_tag(member))?;
    match member {
        ApplicationSchemaMember::Entity { .. }
        | ApplicationSchemaMember::Aspect { .. }
        | ApplicationSchemaMember::Field { .. }
        | ApplicationSchemaMember::Relation { .. }
        | ApplicationSchemaMember::PrincipalBinding { .. } => schema::write(output, member),
        ApplicationSchemaMember::ApplicationQuery { .. }
        | ApplicationSchemaMember::ApplicationCapability { .. }
        | ApplicationSchemaMember::ApplicationCapabilityContext { .. }
        | ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot { .. }
        | ApplicationSchemaMember::ApplicationCapabilityProvenance { .. } => {
            capability::write(output, member)
        }
        ApplicationSchemaMember::Operation { .. }
        | ApplicationSchemaMember::OperationProgram { .. }
        | ApplicationSchemaMember::OperationDecisionRead { .. }
        | ApplicationSchemaMember::OperationMutationPrecondition { .. }
        | ApplicationSchemaMember::OperationDecisionFactBudget { .. }
        | ApplicationSchemaMember::OperationProjectionWorkBudget { .. }
        | ApplicationSchemaMember::OperationExternalEffect { .. }
        | ApplicationSchemaMember::OperationAftermath { .. } => operation::write(output, member),
        ApplicationSchemaMember::Policy { .. }
        | ApplicationSchemaMember::Ability { .. }
        | ApplicationSchemaMember::OperationAbility { .. }
        | ApplicationSchemaMember::AbilityPolicy { .. } => authorization::write(output, member),
        ApplicationSchemaMember::Unit { .. } | ApplicationSchemaMember::Effect { .. } => {
            vocabulary::write(output, member)
        }
    }
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    match input.u16()? {
        tag @ 1..=5 | tag @ 25..=26 => schema::decode(tag, input),
        tag @ 6..=10 => capability::decode(tag, input, budget),
        tag @ 11..=18 => operation::decode(tag, input, budget),
        tag @ 19..=22 => authorization::decode(tag, input, budget),
        tag @ 23..=24 => vocabulary::decode(tag, input),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

pub(super) fn member_tag(member: &ApplicationSchemaMember) -> u16 {
    match member {
        ApplicationSchemaMember::Entity { .. } => 1,
        ApplicationSchemaMember::Aspect { .. } => 2,
        ApplicationSchemaMember::Field { frame: Some(_), .. } => 25,
        ApplicationSchemaMember::Field { .. } => 3,
        ApplicationSchemaMember::Relation { integrity, .. }
            if *integrity
                != worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling() =>
        {
            26
        }
        ApplicationSchemaMember::Relation { .. } => 4,
        ApplicationSchemaMember::PrincipalBinding { .. } => 5,
        ApplicationSchemaMember::ApplicationQuery { .. } => 6,
        ApplicationSchemaMember::ApplicationCapability { .. } => 7,
        ApplicationSchemaMember::ApplicationCapabilityContext { .. } => 8,
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot { .. } => 9,
        ApplicationSchemaMember::ApplicationCapabilityProvenance { .. } => 10,
        ApplicationSchemaMember::Operation { .. } => 11,
        ApplicationSchemaMember::OperationProgram { .. } => 12,
        ApplicationSchemaMember::OperationDecisionRead { .. } => 13,
        ApplicationSchemaMember::OperationMutationPrecondition { .. } => 14,
        ApplicationSchemaMember::OperationDecisionFactBudget { .. } => 15,
        ApplicationSchemaMember::OperationProjectionWorkBudget { .. } => 16,
        ApplicationSchemaMember::OperationExternalEffect { .. } => 17,
        ApplicationSchemaMember::OperationAftermath { .. } => 18,
        ApplicationSchemaMember::Policy { .. } => 19,
        ApplicationSchemaMember::Ability { .. } => 20,
        ApplicationSchemaMember::OperationAbility { .. } => 21,
        ApplicationSchemaMember::AbilityPolicy { .. } => 22,
        ApplicationSchemaMember::Unit { .. } => 23,
        ApplicationSchemaMember::Effect { .. } => 24,
    }
}
