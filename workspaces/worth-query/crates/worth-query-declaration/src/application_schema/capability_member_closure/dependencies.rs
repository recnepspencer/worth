use crate::application_capability::{
    ApplicationCapabilityElevationRule, ApplicationCapabilityFieldBinding,
    ApplicationCapabilityFieldDimension, ApplicationCapabilityRelationBinding,
    ApplicationCapabilityRelationDimension, ErasedApplicationCapabilityContract,
};

use super::super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};
use super::declared_dimensions::DeclaredCapabilityDimensions;

pub(super) fn dependencies_are_closed(
    members: &[ApplicationSchemaMember],
    dimensions: &DeclaredCapabilityDimensions<'_>,
    contract: &ErasedApplicationCapabilityContract,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if !dimensions.context_exists(
        contract.constraints().context(),
        contract.constraints().context_type(),
    ) {
        return Err(
            ApplicationSchemaDeclarationDenial::MissingApplicationCapabilityContextDependency,
        );
    }
    if !dimensions.provenance_exists(
        contract.delegation().provenance(),
        contract.delegation().provenance_type(),
    ) {
        return Err(
            ApplicationSchemaDeclarationDenial::MissingApplicationCapabilityProvenanceDependency,
        );
    }
    if operation_exists(members, contract)
        && entity_exists(members, contract.grant_entity())
        && field_exists(members, contract.target().action().field())
        && field_exists(members, contract.target().purpose().field())
        && relation_exists(members, contract.target().resource())
        && field_dimension_exists(members, contract.target().field())
        && relation_dimension_exists(members, contract.target().relation())
        && field_dimension_exists(members, contract.constraints().magnitude())
        && field_exists(
            members,
            contract.constraints().currentness().active_status().field(),
        )
        && tenure_dependencies_exist(members, contract)
        && contract
            .constraints()
            .currentness()
            .validity()
            .bounds()
            .all(|bound| field_exists(members, bound))
        && relation_exists(members, contract.delegation().grantee())
        && delegation_activation_dependencies_exist(members, contract)
        && elevation_dependencies_exist(members, dimensions, contract.elevation())
    {
        Ok(())
    } else {
        Err(ApplicationSchemaDeclarationDenial::MissingApplicationCapabilityDependency)
    }
}

/// Workflow binding and delegation lineage fields, plus every extent-status
/// field, exist exactly when the contract's tenure declares them.
fn tenure_dependencies_exist(
    members: &[ApplicationSchemaMember],
    contract: &ErasedApplicationCapabilityContract,
) -> bool {
    let currentness = contract.constraints().currentness();
    let workflow_exists = currentness.workflow().is_none_or(|workflow| {
        field_exists(members, workflow.grant()) && field_exists(members, workflow.resource())
    });
    let lineage_exists = contract.delegation().lineage().is_none_or(|lineage| {
        relation_exists(members, lineage.parent())
            && relation_exists(members, lineage.grantor())
            && field_exists(members, lineage.limit())
    });
    workflow_exists
        && lineage_exists
        && currentness
            .extent_status()
            .iter()
            .all(|status| field_exists(members, status.field()))
}

fn delegation_activation_dependencies_exist(
    members: &[ApplicationSchemaMember],
    contract: &ErasedApplicationCapabilityContract,
) -> bool {
    let Some(activation) = contract.delegation().activation() else {
        return true;
    };
    operation_binding_exists(members, activation.operation())
        && field_exists(members, activation.identity())
        && activation
            .context_relations()
            .iter()
            .all(|relation| relation_exists(members, relation))
}

fn elevation_dependencies_exist(
    members: &[ApplicationSchemaMember],
    dimensions: &DeclaredCapabilityDimensions<'_>,
    elevation: &ApplicationCapabilityElevationRule,
) -> bool {
    let ApplicationCapabilityElevationRule::Governed(elevation) = elevation else {
        return true;
    };
    let review = elevation.review();
    let lifecycle = elevation.lifecycle();
    [
        elevation.identity(),
        elevation.reason(),
        elevation.status(),
        review.identity(),
        review.kind().field(),
        review.status(),
    ]
    .into_iter()
    .chain(elevation.validity().bounds())
    .chain(
        elevation
            .states()
            .values()
            .into_iter()
            .map(|state| state.field()),
    )
    .chain([review.required().field(), review.completed().field()])
    .all(|field| field_exists(members, field))
        && [
            elevation.requester(),
            elevation.approver(),
            elevation.grant(),
            review.relation(),
            review.scope(),
            review.reviewer(),
        ]
        .into_iter()
        .all(|relation| relation_exists(members, relation))
        && elevation
            .resource_relation()
            .is_none_or(|relation| relation_exists(members, relation))
        && dimensions.entity_slot_exists(lifecycle.elevation_slot())
        && dimensions.entity_slot_exists(lifecycle.review_slot())
        && lifecycle
            .transitions()
            .into_iter()
            .all(|transition| transition_binding_exists(members, transition))
}

fn operation_exists(
    members: &[ApplicationSchemaMember],
    contract: &ErasedApplicationCapabilityContract,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Operation {
                operation,
                input_type,
            } if operation == contract.operation()
                && input_type.as_str() == contract.input_type()
        )
    })
}

fn operation_binding_exists(
    members: &[ApplicationSchemaMember],
    binding: &crate::application_capability::ApplicationCapabilityOperationBinding,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Operation {
                operation,
                input_type,
            } if operation == binding.operation()
                && input_type.as_str() == binding.input_type()
        )
    })
}

fn transition_binding_exists(
    members: &[ApplicationSchemaMember],
    binding: &crate::application_capability::ApplicationCapabilityTransitionBinding,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::ApplicationCapability { contract }
                if contract.name() == binding.capability()
                    && contract.capability_type() == binding.capability_type()
                    && contract.operation() == binding.operation().operation()
                    && contract.operation_type() == binding.operation().operation_type()
                    && contract.input_type() == binding.operation().input_type()
        )
    })
}

fn entity_exists(members: &[ApplicationSchemaMember], entity: &str) -> bool {
    members.iter().any(
        |member| matches!(member, ApplicationSchemaMember::Entity { entity: found } if found == entity),
    )
}

fn field_exists(
    members: &[ApplicationSchemaMember],
    binding: &ApplicationCapabilityFieldBinding,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Field {
                entity,
                aspect,
                field,
                scalar_family,
                value_type,
                ..
            } if entity == binding.entity()
                && aspect == binding.aspect()
                && field == binding.field()
                && *scalar_family == binding.scalar_family()
                && value_type == binding.value_type()
        )
    })
}

fn relation_exists(
    members: &[ApplicationSchemaMember],
    binding: &ApplicationCapabilityRelationBinding,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Relation { relation, from, to, .. }
                if relation == binding.relation()
                    && from == binding.from()
                    && to == binding.to()
        )
    })
}

fn field_dimension_exists(
    members: &[ApplicationSchemaMember],
    dimension: &ApplicationCapabilityFieldDimension,
) -> bool {
    match dimension {
        ApplicationCapabilityFieldDimension::NotApplicable => true,
        ApplicationCapabilityFieldDimension::Bound(field) => field_exists(members, field),
    }
}

fn relation_dimension_exists(
    members: &[ApplicationSchemaMember],
    dimension: &ApplicationCapabilityRelationDimension,
) -> bool {
    match dimension {
        ApplicationCapabilityRelationDimension::NotApplicable => true,
        ApplicationCapabilityRelationDimension::Bound(relation) => {
            relation_exists(members, relation)
        }
    }
}
