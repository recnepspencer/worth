use crate::application_capability::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityDisclosureRule,
    ApplicationCapabilityFieldBinding, ApplicationCapabilityFieldDimension,
    ApplicationCapabilityGraphRule, ApplicationCapabilityOperationBinding,
    ApplicationCapabilityRelationBinding, ApplicationCapabilityRelationDimension,
    ApplicationCapabilityScopeGuard, ErasedApplicationCapabilityContract,
};

use super::identifier_validation::{
    validate_authorization_path, validate_portable_type_identifier, validate_simple_identifier,
};
use super::ApplicationSchemaDeclarationDenial;

pub(super) fn validate_capability_identifiers(
    contract: &ErasedApplicationCapabilityContract,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for value in [
        contract.name(),
        contract.operation(),
        contract.grant_entity(),
        contract.constraints().context(),
        contract.delegation().provenance(),
    ] {
        validate_simple_identifier(value)?;
    }
    for identity in [
        contract.capability_type(),
        contract.operation_type(),
        contract.input_type(),
        contract.constraints().context_type(),
        contract.delegation().provenance_type(),
    ] {
        validate_portable_type_identifier(identity)?;
    }
    validate_field(contract.target().action().field())?;
    validate_relation(contract.target().resource())?;
    if let Some(slot) = contract.target().resource_path().root_slot() {
        validate_context_slot(slot)?;
    }
    for step in contract.target().resource_path().traversals() {
        validate_identifiers([step.relation(), step.from(), step.to()])?;
    }
    validate_relation_dimension(contract.target().relation())?;
    validate_field_dimension(contract.target().field())?;
    validate_field(contract.target().purpose().field())?;
    validate_field_dimension(contract.constraints().magnitude())?;
    let currentness = contract.constraints().currentness();
    validate_field(currentness.active_status().field())?;
    validate_field(currentness.workflow().grant())?;
    validate_field(currentness.workflow().resource())?;
    validate_field(currentness.validity().not_before())?;
    validate_field(currentness.validity().not_after())?;
    validate_relation(contract.delegation().parent())?;
    validate_relation(contract.delegation().grantor())?;
    validate_relation(contract.delegation().grantee())?;
    validate_field(contract.delegation().limit())?;
    if let Some(activation) = contract.delegation().activation() {
        validate_operation_binding(activation.operation())?;
        validate_field(activation.identity())?;
        for relation in activation.context_relations() {
            validate_relation(relation)?;
        }
    }
    if let Some(revocation) = contract.delegation().revocation() {
        validate_operation_binding(revocation.operation())?;
        validate_field(revocation.identity())?;
        validate_value_binding(revocation.revoked_status())?;
    }
    if let Some(elevation) = contract.elevation().definition() {
        validate_elevation(elevation)?;
    }
    validate_composition(contract)
}

fn validate_elevation(
    elevation: &crate::application_capability::ApplicationCapabilityElevationDefinition,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for field in [
        elevation.identity(),
        elevation.reason(),
        elevation.status(),
        elevation.validity().not_before(),
        elevation.validity().not_after(),
    ] {
        validate_field(field)?;
    }
    for state in elevation.states().values() {
        validate_value_binding(state)?;
    }
    for relation in [
        elevation.requester(),
        elevation.approver(),
        elevation.grant(),
    ] {
        validate_relation(relation)?;
    }
    if let Some(relation) = elevation.resource_relation() {
        validate_relation(relation)?;
    }
    let review = elevation.review();
    for relation in [review.relation(), review.scope(), review.reviewer()] {
        validate_relation(relation)?;
    }
    for field in [review.identity(), review.status()] {
        validate_field(field)?;
    }
    for value in [review.kind(), review.required(), review.completed()] {
        validate_value_binding(value)?;
    }
    validate_context_slot(elevation.lifecycle().elevation_slot())?;
    validate_context_slot(elevation.lifecycle().review_slot())?;
    for transition in elevation.lifecycle().transitions() {
        validate_simple_identifier(transition.capability())?;
        validate_portable_type_identifier(transition.capability_type())?;
        validate_operation_binding(transition.operation())?;
        if let Some(effect) = transition.lifecycle_effect() {
            validate_simple_identifier(effect.effect())?;
            validate_simple_identifier(effect.effect_type())?;
            validate_portable_type_identifier(effect.payload_type())?;
        }
    }
    Ok(())
}

fn validate_operation_binding(
    operation: &ApplicationCapabilityOperationBinding,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_simple_identifier(operation.operation())?;
    validate_portable_type_identifier(operation.operation_type())?;
    validate_portable_type_identifier(operation.input_type())
}

fn validate_context_slot(
    slot: &ApplicationCapabilityContextEntitySlotBinding,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_identifiers([slot.context(), slot.slot(), slot.entity()])?;
    validate_portable_type_identifier(slot.context_identity().as_str())?;
    validate_portable_type_identifier(slot.slot_identity().as_str())
}

fn validate_composition(
    contract: &ErasedApplicationCapabilityContract,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let composition = contract.composition();
    validate_graph_rule(composition.decision().allow().graph())?;
    for rule in [
        composition.decision().deny().graph(),
        composition.decision().conflict().graph(),
        composition.actors().separation_of_duty().graph(),
        composition.actors().distinct_actor().graph(),
    ]
    .into_iter()
    .flatten()
    {
        validate_graph_rule(rule)?;
    }
    if let ApplicationCapabilityDisclosureRule::Permit(guards) =
        composition.propagation().disclosure()
    {
        for guard in guards {
            validate_guard(guard)?;
        }
    }
    Ok(())
}

fn validate_graph_rule(
    rule: &ApplicationCapabilityGraphRule,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for requirement in rule.requirements() {
        for clause in requirement.clauses() {
            validate_authorization_path(clause.path())?;
            validate_guard(clause.guard())?;
            for anchor in clause.context_anchors() {
                validate_relation(anchor.relation())?;
                validate_context_slot(anchor.slot())?;
            }
        }
    }
    Ok(())
}

fn validate_identifiers<'a>(
    values: impl IntoIterator<Item = &'a str>,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for value in values {
        validate_simple_identifier(value)?;
    }
    Ok(())
}

fn validate_guard(
    guard: &ApplicationCapabilityScopeGuard,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for requirement in guard.requirements() {
        validate_field(requirement.field())?;
    }
    Ok(())
}

fn validate_field(
    field: &ApplicationCapabilityFieldBinding,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for value in [field.entity(), field.aspect(), field.field()] {
        validate_simple_identifier(value)?;
    }
    validate_portable_type_identifier(field.value_type())
}

fn validate_value_binding(
    binding: &crate::application_capability::ApplicationCapabilityValueBinding,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_field(binding.field())
}

fn validate_relation(
    relation: &ApplicationCapabilityRelationBinding,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for value in [relation.relation(), relation.from(), relation.to()] {
        validate_simple_identifier(value)?;
    }
    Ok(())
}

fn validate_field_dimension(
    dimension: &ApplicationCapabilityFieldDimension,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    match dimension {
        ApplicationCapabilityFieldDimension::NotApplicable => Ok(()),
        ApplicationCapabilityFieldDimension::Bound(field) => validate_field(field),
    }
}

fn validate_relation_dimension(
    dimension: &ApplicationCapabilityRelationDimension,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    match dimension {
        ApplicationCapabilityRelationDimension::NotApplicable => Ok(()),
        ApplicationCapabilityRelationDimension::Bound(relation) => validate_relation(relation),
    }
}
