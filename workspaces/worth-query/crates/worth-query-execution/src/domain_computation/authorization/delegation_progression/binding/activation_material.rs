use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRequestProjection;

use super::{
    delegation_denial, WorthQueryDelegationActivationMaterial, WorthQueryDelegationResolvedRequest,
    WorthQueryInstalledCapabilityPlan, WorthQueryOperationAuthorizationDenial,
};
use crate::domain_computation::authorization::capability_binding_lowering::field_locator;

pub(super) fn collect_activation_material<Schema, Scope, Context>(
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    installed: &WorthQueryInstalledCapabilityPlan,
    proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
    resolved: &WorthQueryDelegationResolvedRequest,
    activation: &crate::domain_computation::authorization::capability_registry::WorthQueryCapabilityDelegationActivationBindings,
) -> Result<WorthQueryDelegationActivationMaterial, WorthQueryOperationAuthorizationDenial>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    validate_field_bindings(runtime, installed, proposed, activation)?;
    let fields = child_fields(installed, proposed, &activation.identity)?;
    let related = match (&installed.delegation().related, resolved.related()) {
        (Some(relation), Some(entity)) => Some((relation.relation_kind(), entity)),
        (None, None) => None,
        _ => return Err(delegation_denial(installed)),
    };
    let activation_context = resolved.activation_context().collect::<Vec<_>>();
    let canonical_activation_context = activation_context
        .iter()
        .copied()
        .map(|(relation, entity)| super::canonical_relation(relation, entity))
        .collect();
    Ok(WorthQueryDelegationActivationMaterial {
        fields,
        related,
        activation_context,
        canonical_activation_context,
    })
}

fn validate_field_bindings<Schema, Scope, Context>(
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    installed: &WorthQueryInstalledCapabilityPlan,
    proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
    activation: &crate::domain_computation::authorization::capability_registry::WorthQueryCapabilityDelegationActivationBindings,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    let graph = runtime
        .runtime
        .primary_graph()
        .ok_or_else(|| delegation_denial(installed))?;
    let layout = graph.layout();
    let bindings = [
        (proposed.child_identity().field(), &activation.identity),
        (
            proposed.workflow().field(),
            &installed.delegation().grant_workflow,
        ),
        (
            proposed.not_before().field(),
            &installed.delegation().not_before,
        ),
        (
            proposed.not_after().field(),
            &installed.delegation().not_after,
        ),
        (
            proposed.remaining_delegations().field(),
            &installed.delegation().remaining,
        ),
    ];
    if bindings.into_iter().all(|(declared, expected)| {
        field_locator(layout, declared).is_ok_and(|found| &found == expected)
    }) && proposed.not_before().value() <= proposed.not_after().value()
    {
        Ok(())
    } else {
        Err(delegation_denial(installed))
    }
}

fn child_fields<Schema, Scope, Context>(
    installed: &WorthQueryInstalledCapabilityPlan,
    proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
    identity: &AspectFieldLocator,
) -> Result<BTreeMap<AspectFieldLocator, AspectValue>, WorthQueryOperationAuthorizationDenial> {
    let target = proposed.target();
    let mut fields = BTreeMap::from([
        (identity.clone(), proposed.child_identity().value().clone()),
        (
            installed.delegation().action.clone(),
            target.action().clone(),
        ),
        (
            installed.delegation().purpose.clone(),
            target.purpose().clone(),
        ),
        (
            installed.delegation().active_status.0.clone(),
            installed.delegation().active_status.1.clone(),
        ),
        (
            installed.delegation().grant_workflow.clone(),
            proposed.workflow().value().clone(),
        ),
        (
            installed.delegation().not_before.clone(),
            proposed.not_before().value().clone(),
        ),
        (
            installed.delegation().not_after.clone(),
            proposed.not_after().value().clone(),
        ),
        (
            installed.delegation().remaining.clone(),
            proposed.remaining_delegations().value().clone(),
        ),
    ]);
    bind_optional(
        &mut fields,
        installed.delegation().disclosure.as_ref(),
        target.field_value(),
    )?;
    bind_optional(
        &mut fields,
        installed.delegation().magnitude.as_ref(),
        target.magnitude_value(),
    )?;
    let expected = 8
        + usize::from(installed.delegation().disclosure.is_some())
        + usize::from(installed.delegation().magnitude.is_some());
    (fields.len() == expected)
        .then_some(fields)
        .ok_or_else(|| delegation_denial(installed))
}

fn bind_optional(
    fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    locator: Option<&AspectFieldLocator>,
    value: Option<&AspectValue>,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    match (locator, value) {
        (Some(locator), Some(value)) => {
            fields.insert(locator.clone(), value.clone());
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::DelegationRejected),
            "delegated capability optional dimension",
        )),
    }
}
