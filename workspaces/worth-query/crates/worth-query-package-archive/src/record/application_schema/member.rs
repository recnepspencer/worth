use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::decode_budget::RecordDecodeAttempt;

mod authorization;
mod capability;
mod component_set;
mod derived_component_set;
mod derived_relation;
mod history;
mod inspection;
mod invariant;
mod managed_index;
mod mutation_description;
mod operation;
mod projected_member_domain;
mod relation_aspect;
mod relation_graph;
mod resource;
mod resource_value;

mod schema;
mod source_disclosure;
mod vocabulary;

fn require_supported_field_codec(member: &ApplicationSchemaMember) -> Result<(), Denial> {
    managed_index::require_shape(member)?;
    projected_member_domain::require_shape(member)?;
    inspection::require_shape(member)?;
    derived_relation::require_shape(member)?;
    if let ApplicationSchemaMember::DerivedComponentSet { definition } = member {
        derived_component_set::require_shape(definition)?;
    }
    schema::require_supported_field_codec(member)?;
    if let ApplicationSchemaMember::RelationAspect { definition } = member {
        relation_aspect::require_supported_field_codec(definition)?;
    }
    Ok(())
}

pub(super) fn require_encoding_contract(
    member: &ApplicationSchemaMember,
    maximum_depth: u32,
) -> Result<(), Denial> {
    require_supported_field_codec(member)?;
    if let ApplicationSchemaMember::ApplicationResource { definition } = member {
        resource::require_encoding_contract(definition)?;
    }
    if matches!(
        member,
        ApplicationSchemaMember::OperationComponentRead { .. }
    ) {
        super::super::operation_component_read::require_nesting_depth(maximum_depth)?;
    }
    if matches!(
        member,
        ApplicationSchemaMember::ManagedIndex { .. }
            | ApplicationSchemaMember::ApplicationInvariant { .. }
            | ApplicationSchemaMember::ProjectedMemberDomain { .. }
            | ApplicationSchemaMember::ApplicationQueryInspection { .. }
            | ApplicationSchemaMember::DerivedRelation { .. }
            | ApplicationSchemaMember::ComponentSet { .. }
            | ApplicationSchemaMember::DerivedComponentSet { .. }
            | ApplicationSchemaMember::ApplicationMutation { .. }
            | ApplicationSchemaMember::OperationBody { .. }
            | ApplicationSchemaMember::Relation { integrity: worth_query_declaration::facade::application_schema::ApplicationRelationIntegrityRecord { reaches_at_least: Some(_), .. }, .. }
    ) && maximum_depth < 2
    {
        return Err(Denial::new(Kind::NestingDepthBudgetExceeded));
    }
    if matches!(member, ApplicationSchemaMember::OperationAftermath { contract, .. }
        if contract.correction_binding().is_some())
        && maximum_depth < 2
    {
        return Err(Denial::new(Kind::NestingDepthBudgetExceeded));
    }
    if let ApplicationSchemaMember::OperationBody { body, .. } = member {
        crate::record::operation_body::require_nesting_depth(body, maximum_depth)?;
    }
    capability::require_nesting_depth(member, maximum_depth)
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    require_supported_field_codec(member)?;
    if let ApplicationSchemaMember::ApplicationResource { definition } = member {
        resource::require_encoding_contract(definition)?;
    }
    output.u16(member_tag(member))?;
    match member {
        ApplicationSchemaMember::ManagedIndex { definition } => {
            managed_index::write(output, definition)
        }
        ApplicationSchemaMember::DerivedComponentSet { definition } => {
            derived_component_set::write(output, definition)
        }
        ApplicationSchemaMember::ProjectedMemberDomain { definition } => {
            projected_member_domain::write(output, definition)
        }
        ApplicationSchemaMember::DerivedRelation { .. }
        | ApplicationSchemaMember::RuleReadableCollection { .. }
        | ApplicationSchemaMember::OperationDerivedRead { .. } => {
            derived_relation::write(output, member)
        }
        ApplicationSchemaMember::OperationComponentRead { definition } => {
            super::super::operation_component_read::write(output, definition)
        }
        ApplicationSchemaMember::ComponentSet { definition } => {
            component_set::write(output, definition)
        }
        ApplicationSchemaMember::ApplicationResource { definition } => {
            resource::write(output, definition)
        }
        ApplicationSchemaMember::ApplicationMutation { description } => {
            mutation_description::write(output, description)
        }
        ApplicationSchemaMember::ApplicationHistoryQuery { definition } => {
            history::write(output, definition)
        }
        ApplicationSchemaMember::ApplicationQueryInspection { definition } => {
            inspection::write(output, definition)
        }
        ApplicationSchemaMember::RelationAspect { definition } => {
            relation_aspect::write(output, definition)
        }
        ApplicationSchemaMember::OperationBody { operation, body } => {
            output.text(operation)?;
            super::super::operation_body::write(output, body)
        }
        ApplicationSchemaMember::ApplicationInvariant { .. } => invariant::write(output, member),
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
        | ApplicationSchemaMember::OperationProjectionWorkBudget { .. }
        | ApplicationSchemaMember::OperationExternalEffect { .. }
        | ApplicationSchemaMember::OperationInboundOccurrence { .. }
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
        48 => managed_index::decode(input, budget),
        46 => derived_component_set::decode(input, budget),
        47 => derived_relation::decode(47, input, budget),
        45 => inspection::decode(input, budget),
        44 => projected_member_domain::decode(input, budget),
        tag @ 41..=43 => derived_relation::decode(tag, input, budget),
        39 => resource::decode(input, budget)
            .map(|definition| ApplicationSchemaMember::ApplicationResource { definition }),

        36 => history::decode(input, budget)
            .map(|definition| ApplicationSchemaMember::ApplicationHistoryQuery { definition }),
        37 => relation_aspect::decode(input, budget)
            .map(|definition| ApplicationSchemaMember::RelationAspect { definition }),
        40 => Ok(ApplicationSchemaMember::OperationComponentRead {
            definition: super::super::operation_component_read::decode(input, budget)?,
        }),
        38 => component_set::decode(input, budget),
        tag @ 1..=5 | tag @ 25..=26 => schema::decode(tag, input, budget),
        tag @ 6..=10 | tag @ 33 => capability::decode(tag, input, budget),
        tag @ 11..=14 | tag @ 16..=18 | tag @ (29 | 30 | 35) => {
            operation::decode(tag, input, budget)
        }
        tag @ 19..=22 => authorization::decode(tag, input, budget),
        tag @ 23..=24 => vocabulary::decode(tag, input),
        tag @ (27 | 34) => invariant::decode(tag, input, budget),
        31 => mutation_description::decode(input, budget),
        32 => Ok(ApplicationSchemaMember::OperationBody {
            operation: input.text()?.to_owned(),
            body: super::super::operation_body::decode(input, budget)?,
        }),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

pub(super) fn member_tag(member: &ApplicationSchemaMember) -> u16 {
    match member {
        ApplicationSchemaMember::ManagedIndex { .. } => 48,
        ApplicationSchemaMember::ProjectedMemberDomain { .. } => 44,
        ApplicationSchemaMember::ApplicationQueryInspection { .. } => 45,
        ApplicationSchemaMember::DerivedRelation { definition } => if definition.is_legacy_shape() { 41 } else { 47 },
        ApplicationSchemaMember::DerivedComponentSet { .. } => 46,
        ApplicationSchemaMember::RuleReadableCollection { .. } => 42,
        ApplicationSchemaMember::OperationDerivedRead { .. } => 43,
        ApplicationSchemaMember::ApplicationResource { .. } => 39,

        ApplicationSchemaMember::ApplicationHistoryQuery { .. } => 36,
        ApplicationSchemaMember::RelationAspect { .. } => 37,
        ApplicationSchemaMember::OperationComponentRead { .. } => 40,
        ApplicationSchemaMember::ComponentSet { .. } => 38,
        ApplicationSchemaMember::ApplicationMutation { .. } => 31,
        ApplicationSchemaMember::ApplicationInvariant { maximum_work_units: None, .. } => 34,
        ApplicationSchemaMember::ApplicationInvariant { .. } => 27,
        ApplicationSchemaMember::Entity { .. } => 1,
        ApplicationSchemaMember::Aspect { .. } => 2,
        ApplicationSchemaMember::Field { frame: Some(_), .. } => 25,
        ApplicationSchemaMember::Field { .. } => 3,
        ApplicationSchemaMember::Relation { integrity, .. }
            if *integrity
                != worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().into() =>
        {
            26
        }
        ApplicationSchemaMember::Relation { .. } => 4,
        ApplicationSchemaMember::PrincipalBinding { .. } => 5,
        ApplicationSchemaMember::ApplicationQuery { .. } => 6,
        ApplicationSchemaMember::ApplicationCapability { contract }
            if !contract.target().resource_path().is_direct() => 33,
        ApplicationSchemaMember::ApplicationCapability { .. } => 7,
        ApplicationSchemaMember::ApplicationCapabilityContext { .. } => 8,
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot { .. } => 9,
        ApplicationSchemaMember::ApplicationCapabilityProvenance { .. } => 10,
        ApplicationSchemaMember::Operation { .. } => 11,
        ApplicationSchemaMember::OperationProgram { .. } => 12,
        ApplicationSchemaMember::OperationDecisionRead { .. } => 13,
        ApplicationSchemaMember::OperationMutationPrecondition { .. } => 14,
        ApplicationSchemaMember::OperationProjectionWorkBudget { .. } => 16,
        ApplicationSchemaMember::OperationExternalEffect { .. } => 17,
        ApplicationSchemaMember::OperationInboundOccurrence { .. } => 30,
        ApplicationSchemaMember::OperationAftermath { contract, .. }
            if contract.correction_binding().is_some() => 35,
        ApplicationSchemaMember::OperationAftermath { .. } => 18,
        ApplicationSchemaMember::OperationBody { .. } => 32,
        ApplicationSchemaMember::Policy { .. } => 19,
        ApplicationSchemaMember::Ability { .. } => 20,
        ApplicationSchemaMember::OperationAbility { .. } => 21,
        ApplicationSchemaMember::AbilityPolicy { .. } => 22,
        ApplicationSchemaMember::Unit { .. } => 23,
        ApplicationSchemaMember::Effect { .. } => 24,
    }
}
