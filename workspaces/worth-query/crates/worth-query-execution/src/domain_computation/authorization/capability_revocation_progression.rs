//! Command/target binding for one exact capability revocation.

use worth_foundational::facade::{AspectFieldLocator, AspectValue, CanonicalDigestWorkBudget};
use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityRequest, ApplicationCapabilityRevocationRequest,
        ErasedApplicationCapabilityEntitySelector,
    },
    application_schema::TypedMutationPreconditions,
};
use worth_query_installation::facade::{
    derive_capability_revocation_proposal_identity, ApplicationSchema,
    WorthQueryCanonicalWorkEvidence, WorthQueryCapabilityRevocationProposalBasis,
    WorthQueryInstalledApplicationCapability, WorthQueryInstalledApplicationOperation,
    WorthQueryOperationFieldTouchScope, WorthQueryOperationTouchScope,
};
use worth_relational::facade::identity::{EntityId, KindId};

use super::operation_progression::{
    progress_capability_operation, WorthQueryCapabilityOperationProgression,
};
use super::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(in crate::domain_computation) struct WorthQueryCapabilityRevocationBinding {
    proposal_identity: [u8; 32],
    required_field_touch: WorthQueryOperationFieldTouchScope,
    target_kind: KindId,
    target_entity: String,
    resource: EntityId,
    resource_relation: KindId,
    identity: AspectFieldLocator,
    identity_value: AspectValue,
    status: AspectFieldLocator,
    active: AspectValue,
    revoked: AspectValue,
}

impl WorthQueryCapabilityRevocationBinding {
    pub(in crate::domain_computation) const fn proposal_identity(&self) -> &[u8; 32] {
        &self.proposal_identity
    }
    pub(in crate::domain_computation) const fn required_field_touch(
        &self,
    ) -> &WorthQueryOperationFieldTouchScope {
        &self.required_field_touch
    }
    pub(in crate::domain_computation) const fn target_kind(&self) -> KindId {
        self.target_kind
    }
    pub(in crate::domain_computation) fn target_entity(&self) -> &str {
        &self.target_entity
    }
    pub(in crate::domain_computation) const fn resource(&self) -> EntityId {
        self.resource
    }
    pub(in crate::domain_computation) const fn resource_relation(&self) -> KindId {
        self.resource_relation
    }
    pub(in crate::domain_computation) const fn identity(&self) -> &AspectFieldLocator {
        &self.identity
    }
    pub(in crate::domain_computation) const fn identity_value(&self) -> &AspectValue {
        &self.identity_value
    }
    pub(in crate::domain_computation) const fn status(&self) -> &AspectFieldLocator {
        &self.status
    }
    pub(in crate::domain_computation) const fn active(&self) -> &AspectValue {
        &self.active
    }
    pub(in crate::domain_computation) const fn revoked(&self) -> &AspectValue {
        &self.revoked
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn authorize_capability_revocation<Capability, Operation, Input>(
        &self,
        access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation:
            worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            >,
        Input: ApplicationCapabilityRequest<Schema, Capability>
            + ApplicationCapabilityRevocationRequest<Schema, Capability>,
    {
        let projection = access
            .capability_input()
            .capability_revocation_target()
            .map_err(|denial| rejected(denial.subject()))?;
        let installed = self
            .authorization
            .capability_plan(capability)
            .ok_or_else(|| rejected(capability.contract().name()))?;
        let revocation = installed
            .delegation()
            .revocation
            .as_ref()
            .ok_or_else(|| rejected(capability.contract().name()))?;
        validate_operation::<Operation, Input>(revocation, operation)?;
        validate_selector(projection.target(), capability, &revocation.identity)?;
        let required_field_touch = operation
            .contracts()
            .touches()
            .scopes()
            .iter()
            .find_map(|scope| match scope {
                WorthQueryOperationTouchScope::WriteField(scope) => Some(scope.clone()),
                _ => None,
            })
            .ok_or_else(|| rejected(operation.operation()))?;
        let budget = operation
            .contracts()
            .capability_revocation_proposal_canonical_work_budget()
            .ok_or_else(|| rejected(operation.operation()))?;
        let (proposal_identity, work) = proposal_identity(
            *capability.identity().bytes(),
            access.resource_entity_id(),
            installed.grant_kind(),
            installed.delegation().resource.relation_kind(),
            projection.target(),
            &revocation.revoked_status.0,
            &installed.delegation().active_status.1,
            &revocation.revoked_status.1,
            budget,
        )?;
        let identity_value = projection.target().value().clone();
        let target_entity = projection.target().entity().to_owned();
        let resource = access.resource_entity_id();
        let admitted = progress_capability_operation(
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::CapabilityRevocation,
        )?;
        admitted.bind_capability_revocation(
            WorthQueryCapabilityRevocationBinding {
                proposal_identity,
                required_field_touch,
                target_kind: installed.grant_kind(),
                target_entity,
                resource,
                resource_relation: installed.delegation().resource.relation_kind(),
                identity: revocation.identity.clone(),
                identity_value,
                status: revocation.revoked_status.0.clone(),
                active: installed.delegation().active_status.1.clone(),
                revoked: revocation.revoked_status.1.clone(),
            },
            work,
        )
    }
}

fn validate_operation<Operation, Input>(
    revocation: &super::capability_registry::WorthQueryCapabilityRevocationBindings,
    operation: &WorthQueryInstalledApplicationOperation<impl ApplicationSchema, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if revocation.operation == operation.operation()
        && revocation.operation_type == operation.operation()
        && revocation.input_type == operation.input_type()
    {
        Ok(())
    } else {
        Err(rejected(operation.operation()))
    }
}

fn validate_selector<Schema, Capability, Operation, Input>(
    selector: &ErasedApplicationCapabilityEntitySelector,
    capability: &WorthQueryInstalledApplicationCapability<Schema, Capability, Operation, Input>,
    identity: &AspectFieldLocator,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    let declared = capability
        .contract()
        .delegation()
        .revocation()
        .ok_or_else(|| rejected(capability.contract().name()))?
        .identity();
    let exact = selector.entity() == declared.entity()
        && selector.aspect() == declared.aspect()
        && selector.field() == declared.field()
        && selector.scalar_family() == declared.scalar_family()
        && selector.value_type() == declared.value_type()
        && identity.field_path().fields().len() == 1;
    exact
        .then_some(())
        .ok_or_else(|| rejected("capability revocation target"))
}

#[allow(clippy::too_many_arguments)]
fn proposal_identity(
    capability: [u8; 32],
    resource: EntityId,
    target_kind: KindId,
    resource_relation: KindId,
    selector: &ErasedApplicationCapabilityEntitySelector,
    status: &AspectFieldLocator,
    active: &AspectValue,
    revoked: &AspectValue,
    budget: CanonicalDigestWorkBudget,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), WorthQueryOperationAuthorizationDenial> {
    derive_capability_revocation_proposal_identity(
        WorthQueryCapabilityRevocationProposalBasis {
            capability,
            resource: (
                resource.partition_id.0,
                resource.local_slot.0,
                resource.generation.0,
            ),
            target_kind: target_kind.as_u32(),
            resource_relation: resource_relation.as_u32(),
            target_entity: selector.entity(),
            target_aspect: selector.aspect(),
            target_field: selector.field(),
            target_value_type: selector.value_type(),
            target_value: selector.value(),
            status_aspect: status.aspect().aspect_key().as_str(),
            status_field: status.field_path().fields()[0].as_str(),
            active,
            revoked,
        },
        budget,
    )
    .map_err(|_| rejected("revocation proposal"))
}

fn rejected(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::DelegationRejected,
        subject,
    )
}
