//! Atomic delegation resolution and exact parent-support observation.

use std::sync::Arc;

use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRequestProjection;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
};
use worth_relational::facade::authorization::{
    RelationalAuthorizationFieldComparison as Comparison, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPathPlan, RelationalAuthorizationPredicate,
    RelationalAuthorizationRelatedEntityConstraint, RelationalAuthorizationTraversal,
};
use worth_relational::facade::identity::EntityId;

use super::WorthQueryExactCapabilityObservation;
use crate::domain_computation::authorization::capability_binding_lowering::relation;
use crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan;
use crate::domain_computation::authorization::delegation_admission::WorthQueryCapabilityObservationSource;
use crate::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRetainedCapabilitySupport,
};

pub(in crate::domain_computation::authorization) struct WorthQueryDelegationResolvedRequest {
    parent: EntityId,
    grantor: EntityId,
    grantee: EntityId,
    resource: EntityId,
    related: Option<EntityId>,
    activation_context: Vec<WorthQueryResolvedActivationContext>,
}

struct WorthQueryResolvedActivationContext {
    traversal: RelationalAuthorizationTraversal,
    entity: EntityId,
}

impl WorthQueryDelegationResolvedRequest {
    pub(in crate::domain_computation::authorization) const fn parent(&self) -> EntityId {
        self.parent
    }

    pub(in crate::domain_computation::authorization) const fn grantor(&self) -> EntityId {
        self.grantor
    }

    pub(in crate::domain_computation::authorization) const fn grantee(&self) -> EntityId {
        self.grantee
    }

    pub(in crate::domain_computation::authorization) const fn resource(&self) -> EntityId {
        self.resource
    }

    pub(in crate::domain_computation::authorization) const fn related(&self) -> Option<EntityId> {
        self.related
    }

    pub(in crate::domain_computation::authorization) fn activation_context(
        &self,
    ) -> impl Iterator<Item = (worth_relational::facade::identity::KindId, EntityId)> + '_ {
        self.activation_context
            .iter()
            .map(|context| (context.traversal.relation_kind(), context.entity))
    }
}

impl<Schema> WorthQueryExactCapabilityObservation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    /// Resolves stable proposal identity for an already-committed retry. This
    /// never supplies parent-currentness or narrowing evidence for a new effect.
    pub(in crate::domain_computation::authorization) fn resolve_delegation_replay_target<
        TargetCapability,
        TargetOperation,
        TargetInput,
        Scope,
        Context,
    >(
        &self,
        target_capability: &WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
    ) -> Result<WorthQueryDelegationResolvedRequest, WorthQueryOperationAuthorizationDenial> {
        let installed = self
            .runtime
            .authorization
            .capability_plan(target_capability)
            .ok_or_else(|| stale(target_capability.contract().name()))?;
        let target = self.resolve_request(proposed.target())?;
        let parent = self.resolve_selector(proposed.parent())?;
        let grantee = self.resolve_selector(proposed.grantee())?;
        if parent.entity_kind() != installed.grant_kind()
            || grantee.entity_kind() != installed.principal_kind()
        {
            return Err(rejected(installed));
        }
        let activation_context = self.resolve_activation_context(installed, proposed)?;
        Ok(WorthQueryDelegationResolvedRequest {
            parent: parent.entity_id(),
            grantor: self.principal,
            grantee: grantee.entity_id(),
            resource: target.resource_entity_id(),
            related: target.related(),
            activation_context,
        })
    }

    pub(in crate::domain_computation::authorization) fn authorize_delegation_support<
        TargetCapability,
        TargetOperation,
        TargetInput,
        Scope,
        Context,
    >(
        &self,
        target_capability: &WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
    ) -> Result<crate::domain_computation::authorization::delegation_progression::support::WorthQueryObservedDelegationSupport, WorthQueryOperationAuthorizationDenial>{
        let installed = self
            .runtime
            .authorization
            .capability_plan(target_capability)
            .ok_or_else(|| stale(target_capability.contract().name()))?;
        let sample = self.runtime.sample_capability_time(installed)?;
        let target = self.resolve_request(proposed.target())?;
        let parent = self.resolve_selector(proposed.parent())?;
        let grantee = self.resolve_selector(proposed.grantee())?;
        if parent.entity_kind() != installed.grant_kind()
            || grantee.entity_kind() != installed.principal_kind()
        {
            return Err(rejected(installed));
        }
        let activation_context = self.resolve_activation_context(installed, proposed)?;
        let retained = WorthQueryRetainedCapabilityRequest::capture(
            *target_capability.identity().bytes(),
            self.principal,
            proposed.target(),
            &target,
        );
        let resolved = WorthQueryDelegationResolvedRequest {
            parent: parent.entity_id(),
            grantor: self.principal,
            grantee: grantee.entity_id(),
            resource: target.resource_entity_id(),
            related: target.related(),
            activation_context,
        };
        let observation = self.bind_capability_observation(installed, &retained, &sample)?;
        let mut decision = observation
            .observe_active_capability(Some(resolved.parent), None)?
            .into_decision_for_grant(resolved.parent)
            .map_err(|()| rejected(installed))?;
        let narrowing = self.observe_narrowing(installed, proposed, &resolved)?;
        decision
            .attach_delegation_activation(
                crate::domain_computation::authorization::decision_facts::WorthQueryDelegationActivationDecisionFact::new(
                    self.session,
                    narrowing,
                ),
            )
            .map_err(|()| inconsistent(installed.contract().name()))?;
        let supporting = WorthQueryRetainedCapabilitySupport::active(
            decision,
            Arc::clone(installed.capability_authority_identity()),
            resolved.parent,
            retained,
            sample,
        );
        Ok(crate::domain_computation::authorization::delegation_progression::support::WorthQueryObservedDelegationSupport::new(resolved, supporting))
    }

    fn resolve_activation_context<Scope, Context>(
        &self,
        installed: &WorthQueryInstalledCapabilityPlan,
        proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
    ) -> Result<Vec<WorthQueryResolvedActivationContext>, WorthQueryOperationAuthorizationDenial>
    {
        let expected = installed
            .delegation()
            .activation
            .as_ref()
            .ok_or_else(|| rejected(installed))?
            .context_relations
            .as_slice();
        if proposed.activation_context().len() != expected.len() {
            return Err(rejected(installed));
        }
        let mut resolved = Vec::with_capacity(expected.len());
        let mut relation_kinds = std::collections::BTreeSet::new();
        for selected in proposed.activation_context() {
            let traversal = relation(
                self.layout,
                selected.relation(),
                worth_relational::facade::authorization::RelationalAuthorizationTraversalDirection::Forward,
            )?;
            if traversal.from_kind() != installed.grant_kind()
                || !expected.iter().any(|candidate| candidate == &traversal)
                || !relation_kinds.insert(traversal.relation_kind())
            {
                return Err(rejected(installed));
            }
            let entity = self.resolve_selector(selected.selector())?;
            if entity.entity_kind() != traversal.to_kind() {
                return Err(rejected(installed));
            }
            resolved.push(WorthQueryResolvedActivationContext {
                traversal,
                entity: entity.entity_id(),
            });
        }
        resolved.sort_by_key(|context| context.traversal.relation_kind());
        Ok(resolved)
    }

    fn observe_narrowing<Scope, Context>(
        &self,
        installed: &WorthQueryInstalledCapabilityPlan,
        proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
        resolved: &WorthQueryDelegationResolvedRequest,
    ) -> Result<
        worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
        WorthQueryOperationAuthorizationDenial,
    > {
        let mut related = vec![RelationalAuthorizationRelatedEntityConstraint::new(
            0,
            installed.delegation().grantee_from_grant.clone(),
            resolved.grantor,
        )];
        related.extend(resolved.activation_context.iter().map(|context| {
            RelationalAuthorizationRelatedEntityConstraint::new(
                0,
                context.traversal.clone(),
                context.entity,
            )
        }));
        let path = RelationalAuthorizationPathPlan::new([], predicates(installed, proposed))
            .with_related_entities(related);
        let plan = RelationalAuthorizationObservationPlan::try_new(
            self.snapshot.clone(),
            resolved.parent,
            resolved.parent,
            installed.grant_kind(),
            installed.grant_kind(),
            [path],
            [],
        )
        .map_err(|_| rejected(installed))?;
        let evidence = self
            .relational
            .observe_authorization(plan)
            .map_err(|_| rejected(installed))?;
        let [path] = evidence.paths() else {
            return Err(rejected(installed));
        };
        (path.matched() && path.exhaustive() && evidence.counters().maximum_frontier_width <= 1)
            .then_some(evidence)
            .ok_or_else(|| rejected(installed))
    }
}

fn predicates<Schema, Scope, Context>(
    installed: &WorthQueryInstalledCapabilityPlan,
    proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
) -> Vec<RelationalAuthorizationPredicate> {
    let target = proposed.target();
    let mut predicates = vec![
        predicate(
            installed,
            &installed.delegation().action,
            Comparison::Equal,
            target.action(),
        ),
        predicate(
            installed,
            &installed.delegation().purpose,
            Comparison::Equal,
            target.purpose(),
        ),
        predicate(
            installed,
            &installed.delegation().active_status.0,
            Comparison::Equal,
            &installed.delegation().active_status.1,
        ),
        predicate(
            installed,
            &installed.delegation().grant_workflow,
            Comparison::Equal,
            proposed.workflow().value(),
        ),
        predicate(
            installed,
            &installed.delegation().not_before,
            Comparison::AtMost,
            proposed.not_before().value(),
        ),
        predicate(
            installed,
            &installed.delegation().not_after,
            Comparison::AtLeast,
            proposed.not_after().value(),
        ),
        predicate(
            installed,
            &installed.delegation().remaining,
            Comparison::StrictlyGreater,
            proposed.remaining_delegations().value(),
        ),
    ];
    if let (Some(field), Some(value)) = (&installed.delegation().disclosure, target.field_value()) {
        predicates.push(predicate(installed, field, Comparison::Equal, value));
    }
    if let (Some(field), Some(value)) =
        (&installed.delegation().magnitude, target.magnitude_value())
    {
        predicates.push(predicate(installed, field, Comparison::AtLeast, value));
    }
    predicates
}

fn predicate(
    installed: &WorthQueryInstalledCapabilityPlan,
    field: &worth_foundational::facade::AspectFieldLocator,
    comparison: Comparison,
    value: &worth_foundational::facade::AspectValue,
) -> RelationalAuthorizationPredicate {
    RelationalAuthorizationPredicate::compare(
        0,
        installed.grant_kind(),
        field.clone(),
        comparison,
        value.clone(),
    )
}

fn rejected(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::DelegationRejected,
        installed.contract().name(),
    )
}

fn stale(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
        subject,
    )
}

fn inconsistent(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
        subject,
    )
}
