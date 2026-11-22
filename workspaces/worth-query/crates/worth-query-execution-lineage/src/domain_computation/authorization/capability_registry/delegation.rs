use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityDelegationRule, ApplicationCapabilityRelationDimension,
    ErasedApplicationCapabilityContract,
};
use worth_relational::facade::authorization::{
    RelationalAuthorizationTraversal, RelationalAuthorizationTraversalDirection,
};

use super::field_binding;
use crate::domain_computation::authorization::capability_binding_lowering::{
    field_locator, relation,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

pub struct WorthQueryCapabilityDelegationBindings {
    pub rule: ApplicationCapabilityDelegationRule,
    pub parent: RelationalAuthorizationTraversal,
    pub grantor: RelationalAuthorizationTraversal,
    pub grantee: RelationalAuthorizationTraversal,
    pub grantee_from_grant: RelationalAuthorizationTraversal,
    pub resource: RelationalAuthorizationTraversal,
    pub related: Option<RelationalAuthorizationTraversal>,
    pub action: AspectFieldLocator,
    pub purpose: AspectFieldLocator,
    pub disclosure: Option<AspectFieldLocator>,
    pub magnitude: Option<AspectFieldLocator>,
    pub active_status: (AspectFieldLocator, AspectValue),
    pub grant_workflow: AspectFieldLocator,
    pub resource_workflow: AspectFieldLocator,
    pub not_before: AspectFieldLocator,
    pub not_after: AspectFieldLocator,
    pub remaining: AspectFieldLocator,
    pub activation: Option<WorthQueryCapabilityDelegationActivationBindings>,
    pub revocation: Option<WorthQueryCapabilityRevocationBindings>,
}

pub struct WorthQueryCapabilityDelegationActivationBindings {
    pub operation: String,
    pub operation_type: String,
    pub input_type: String,
    pub identity: AspectFieldLocator,
    pub context_relations: Vec<RelationalAuthorizationTraversal>,
}

pub struct WorthQueryCapabilityRevocationBindings {
    pub operation: String,
    pub operation_type: String,
    pub input_type: String,
    pub identity: AspectFieldLocator,
    pub revoked_status: (AspectFieldLocator, AspectValue),
}

impl WorthQueryCapabilityDelegationBindings {
    pub fn compile(
        contract: &ErasedApplicationCapabilityContract,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        let delegation = contract.delegation();
        let currentness = contract.constraints().currentness();
        let workflow = currentness.workflow();
        let validity = currentness.validity();
        let active_status = (
            field_locator(layout, currentness.active_status().field())?,
            currentness.active_status().value().clone(),
        );
        Ok(Self {
            rule: contract.composition().propagation().delegation(),
            parent: relation(
                layout,
                delegation.parent(),
                RelationalAuthorizationTraversalDirection::Forward,
            )?,
            grantor: relation(
                layout,
                delegation.grantor(),
                RelationalAuthorizationTraversalDirection::Reverse,
            )?,
            grantee: relation(
                layout,
                delegation.grantee(),
                RelationalAuthorizationTraversalDirection::Forward,
            )?,
            grantee_from_grant: relation(
                layout,
                delegation.grantee(),
                RelationalAuthorizationTraversalDirection::Reverse,
            )?,
            resource: relation(
                layout,
                contract.target().resource(),
                RelationalAuthorizationTraversalDirection::Forward,
            )?,
            related: match contract.target().relation() {
                ApplicationCapabilityRelationDimension::NotApplicable => None,
                ApplicationCapabilityRelationDimension::Bound(binding) => Some(relation(
                    layout,
                    binding,
                    RelationalAuthorizationTraversalDirection::Forward,
                )?),
            },
            action: field_locator(layout, contract.target().action().field())?,
            purpose: field_locator(layout, contract.target().purpose().field())?,
            disclosure: field_binding(contract.target().field())
                .map(|binding| field_locator(layout, binding))
                .transpose()?,
            magnitude: field_binding(contract.constraints().magnitude())
                .map(|binding| field_locator(layout, binding))
                .transpose()?,
            active_status: active_status.clone(),
            grant_workflow: field_locator(layout, workflow.grant())?,
            resource_workflow: field_locator(layout, workflow.resource())?,
            not_before: field_locator(layout, validity.not_before())?,
            not_after: field_locator(layout, validity.not_after())?,
            remaining: field_locator(layout, delegation.limit())?,
            activation: delegation
                .activation()
                .map(|activation| {
                    Ok(WorthQueryCapabilityDelegationActivationBindings {
                        operation: activation.operation().operation().to_string(),
                        operation_type: activation.operation().operation_type().to_string(),
                        input_type: activation.operation().input_type().to_string(),
                        identity: field_locator(layout, activation.identity())?,
                        context_relations: activation
                            .context_relations()
                            .iter()
                            .map(|binding| {
                                relation(
                                    layout,
                                    binding,
                                    RelationalAuthorizationTraversalDirection::Forward,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    })
                })
                .transpose()?,
            revocation: delegation
                .revocation()
                .map(|revocation| {
                    let revoked_status = (
                        field_locator(layout, revocation.revoked_status().field())?,
                        revocation.revoked_status().value().clone(),
                    );
                    if revoked_status.0 != active_status.0 || revoked_status.1 == active_status.1 {
                        return Err(WorthQueryOperationAuthorizationDenial::new(
                            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::InconsistentDecision),
                            contract.name(),
                        ));
                    }
                    Ok(WorthQueryCapabilityRevocationBindings {
                        operation: revocation.operation().operation().to_string(),
                        operation_type: revocation.operation().operation_type().to_string(),
                        input_type: revocation.operation().input_type().to_string(),
                        identity: field_locator(layout, revocation.identity())?,
                        revoked_status,
                    })
                })
                .transpose()?,
        })
    }
}
