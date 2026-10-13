use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRequestProjection;
use worth_query_declaration::facade::application_schema::ApplicationOperationProgramTarget;
use worth_query_installation::facade::{
    derive_delegation_proposal_identity, WorthQueryDelegationProposalIdentityBasis,
    WorthQueryInstalledApplicationCapability, WorthQueryInstalledApplicationOperation,
};
use worth_relational::facade::identity::{EntityId, KindId};

use super::super::capability_registry::WorthQueryInstalledCapabilityPlan;
use super::denial;
use super::support::WorthQueryDelegationResolvedRequest;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

mod activation_material;
mod program;
use activation_material::collect_activation_material;
pub(in crate::domain_computation) use program::WorthQueryDelegationActivationEffect;

pub(in crate::domain_computation) struct WorthQueryDelegationActivationBinding {
    proposal_identity: [u8; 32],
    required_program_targets: Vec<ApplicationOperationProgramTarget>,
    child_kind: KindId,
    child_key: String,
    fields: BTreeMap<AspectFieldLocator, AspectValue>,
    parent_relation: KindId,
    grantor_relation: KindId,
    grantee_relation: KindId,
    resource_relation: KindId,
    related_relation: Option<KindId>,
    parent: EntityId,
    grantor: EntityId,
    grantee: EntityId,
    resource: EntityId,
    related: Option<EntityId>,
    activation_context: Vec<(KindId, EntityId)>,
}

impl WorthQueryDelegationActivationBinding {
    pub(in crate::domain_computation) const fn proposal_identity(&self) -> &[u8; 32] {
        &self.proposal_identity
    }
}

pub(super) struct WorthQueryPreparedDelegationActivation {
    binding: WorthQueryDelegationActivationBinding,
    canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryPreparedDelegationActivation {
    pub(super) fn finish<Schema, Operation, Input, Scope>(
        self,
        mut admitted: crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    > {
        admitted.retain_delegation_proposal_canonical_work(self.canonical_work);
        admitted.bind_delegation_activation(self.binding)
    }
}

struct WorthQueryDelegationActivationAuthority<'a> {
    target_capability_identity: [u8; 32],
    required_program_targets: &'a [ApplicationOperationProgramTarget],
    proposal_budget: worth_foundational::facade::CanonicalDigestWorkBudget,
}

struct WorthQueryDelegationActivationMaterial {
    fields: BTreeMap<AspectFieldLocator, AspectValue>,
    related: Option<(KindId, EntityId)>,
    activation_context: Vec<(KindId, EntityId)>,
    canonical_activation_context: Vec<(u32, (u32, u64, u32))>,
}

impl<'a> WorthQueryDelegationActivationAuthority<'a> {
    fn from_validated_context<
        Schema,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
    >(
        target: &'a WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    ) -> Option<Self>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        Some(Self {
            target_capability_identity: *target.identity().bytes(),
            required_program_targets: target.delegation_activation_program_targets()?,
            proposal_budget: operation
                .contracts()
                .delegation_activation_proposal_canonical_work_budget()?,
        })
    }
}

pub(super) fn bind_activation<
    Schema,
    TargetCapability,
    TargetOperation,
    TargetInput,
    Operation,
    Input,
    Scope,
    Context,
>(
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    installed: &WorthQueryInstalledCapabilityPlan,
    proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
    resolved: WorthQueryDelegationResolvedRequest,
    target: &WorthQueryInstalledApplicationCapability<
        Schema,
        TargetCapability,
        TargetOperation,
        TargetInput,
    >,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<WorthQueryPreparedDelegationActivation, WorthQueryOperationAuthorizationDenial>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    let authority =
        WorthQueryDelegationActivationAuthority::from_validated_context(target, operation)
            .ok_or_else(|| delegation_denial(installed))?;
    let activation = installed
        .delegation()
        .activation
        .as_ref()
        .ok_or_else(|| delegation_denial(installed))?;
    let material =
        collect_activation_material(runtime, installed, proposed, &resolved, activation)?;
    let (proposal_identity, canonical_work) =
        derive_activation_proposal(&authority, installed, proposed, &resolved, &material)?;
    let binding = material.into_binding(
        installed,
        proposed,
        &resolved,
        &authority,
        proposal_identity,
    );
    Ok(WorthQueryPreparedDelegationActivation {
        binding,
        canonical_work,
    })
}

fn derive_activation_proposal<Schema, Scope, Context>(
    authority: &WorthQueryDelegationActivationAuthority<'_>,
    installed: &WorthQueryInstalledCapabilityPlan,
    proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
    resolved: &WorthQueryDelegationResolvedRequest,
    material: &WorthQueryDelegationActivationMaterial,
) -> Result<
    (
        [u8; 32],
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ),
    WorthQueryOperationAuthorizationDenial,
> {
    let (proposal_identity, canonical_work) = derive_delegation_proposal_identity(
        WorthQueryDelegationProposalIdentityBasis {
            target_capability_identity: authority.target_capability_identity,
            child_kind: installed.grant_kind().as_u32(),
            child_key: proposed.child_key(),
            fields: &material.fields,
            parent: canonical_relation(
                installed.delegation().parent.relation_kind(),
                resolved.parent(),
            ),
            grantor: canonical_relation(
                installed.delegation().grantor.relation_kind(),
                resolved.grantor(),
            ),
            grantee: canonical_relation(
                installed.delegation().grantee.relation_kind(),
                resolved.grantee(),
            ),
            resource: canonical_relation(
                installed.delegation().resource.relation_kind(),
                resolved.resource(),
            ),
            related: material
                .related
                .map(|(relation, entity)| canonical_relation(relation, entity)),
            activation_context: &material.canonical_activation_context,
        },
        authority.proposal_budget,
    )
    .map_err(|_| delegation_denial(installed))?;
    Ok((proposal_identity, canonical_work))
}

impl WorthQueryDelegationActivationMaterial {
    fn into_binding<Schema, Scope, Context>(
        self,
        installed: &WorthQueryInstalledCapabilityPlan,
        proposed: &ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
        resolved: &WorthQueryDelegationResolvedRequest,
        authority: &WorthQueryDelegationActivationAuthority<'_>,
        proposal_identity: [u8; 32],
    ) -> WorthQueryDelegationActivationBinding {
        WorthQueryDelegationActivationBinding {
            proposal_identity,
            required_program_targets: authority.required_program_targets.to_vec(),
            child_kind: installed.grant_kind(),
            child_key: proposed.child_key().to_string(),
            fields: self.fields,
            parent_relation: installed.delegation().parent.relation_kind(),
            grantor_relation: installed.delegation().grantor.relation_kind(),
            grantee_relation: installed.delegation().grantee.relation_kind(),
            resource_relation: installed.delegation().resource.relation_kind(),
            related_relation: self.related.map(|(relation, _)| relation),
            parent: resolved.parent(),
            grantor: resolved.grantor(),
            grantee: resolved.grantee(),
            resource: resolved.resource(),
            related: self.related.map(|(_, entity)| entity),
            activation_context: self.activation_context,
        }
    }
}

fn canonical_relation(relation: KindId, entity: EntityId) -> (u32, (u32, u64, u32)) {
    (
        relation.as_u32(),
        (
            entity.partition_id.0,
            entity.local_slot.0,
            entity.generation.0,
        ),
    )
}

fn delegation_denial(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::DelegationRejected,
        ),
        installed.contract().name(),
    )
}
