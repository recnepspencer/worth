use std::{collections::BTreeSet, marker::PhantomData};

use crate::application_schema::ApplicationOperationRef;

use super::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityFieldBinding,
    ApplicationCapabilityOperationBinding, ApplicationCapabilityRelatedEntitySelector,
    ApplicationCapabilityRelationBinding, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityValueBinding, ErasedApplicationCapabilityEntitySelector,
};

mod portable_parts;
pub use portable_parts::{
    WorthQueryPortableApplicationCapabilityDelegationActivationParts,
    WorthQueryPortableApplicationCapabilityRevocationParts,
};

const MAXIMUM_ACTIVATION_CONTEXT_RELATIONS: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityDelegationActivationDefinition {
    operation: ApplicationCapabilityOperationBinding,
    identity: ApplicationCapabilityFieldBinding,
    context_relations: Vec<ApplicationCapabilityRelationBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityRevocationDefinition {
    operation: ApplicationCapabilityOperationBinding,
    identity: ApplicationCapabilityFieldBinding,
    revoked_status: ApplicationCapabilityValueBinding,
}

impl ApplicationCapabilityRevocationDefinition {
    pub fn new<Schema, Operation, Input>(
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        identity: ApplicationCapabilityFieldBinding,
        revoked_status: ApplicationCapabilityValueBinding,
    ) -> Self {
        Self {
            operation: ApplicationCapabilityOperationBinding::from_reference(operation),
            identity,
            revoked_status,
        }
    }

    pub const fn operation(&self) -> &ApplicationCapabilityOperationBinding {
        &self.operation
    }

    pub const fn identity(&self) -> &ApplicationCapabilityFieldBinding {
        &self.identity
    }

    pub const fn revoked_status(&self) -> &ApplicationCapabilityValueBinding {
        &self.revoked_status
    }
}

/// Application-owned selection of the exact capability grant to revoke.
///
/// Command authority remains bound to the ordinary capability request. This
/// projection names only the independently governed transition subject.
pub trait ApplicationCapabilityRevocationRequest<Schema, Capability> {
    fn capability_revocation_target(
        &self,
    ) -> Result<
        ApplicationCapabilityRevocationRequestProjection<Schema>,
        ApplicationCapabilityRevocationRequestProjectionDenial,
    >;
}

pub struct ApplicationCapabilityRevocationRequestProjection<Schema> {
    target: ErasedApplicationCapabilityEntitySelector,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> ApplicationCapabilityRevocationRequestProjection<Schema> {
    pub fn new<Entity>(target: ApplicationCapabilityEntitySelector<Schema, Entity>) -> Self {
        Self {
            target: target.erase(),
            _schema: PhantomData,
        }
    }

    pub const fn target(&self) -> &ErasedApplicationCapabilityEntitySelector {
        &self.target
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationCapabilityRevocationRequestProjectionDenial {
    subject: String,
}

impl ApplicationCapabilityRevocationRequestProjectionDenial {
    pub fn input_variant(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
        }
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl ApplicationCapabilityDelegationActivationDefinition {
    pub fn new<Schema, Operation, Input>(
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        identity: ApplicationCapabilityFieldBinding,
    ) -> Self {
        Self {
            operation: ApplicationCapabilityOperationBinding::from_reference(operation),
            identity,
            context_relations: Vec::new(),
        }
    }

    /// Declares child-to-context relations that belong to every activated grant.
    ///
    /// Target, currentness, and lineage effects remain derived from the
    /// capability contract. These relations are the only application-specific
    /// activation links admitted in addition to that target-owned set.
    pub fn with_context_relations(
        mut self,
        relations: impl IntoIterator<Item = ApplicationCapabilityRelationBinding>,
    ) -> Self {
        self.context_relations = relations.into_iter().collect();
        self.context_relations.sort();
        self.context_relations.dedup();
        self
    }

    pub const fn operation(&self) -> &ApplicationCapabilityOperationBinding {
        &self.operation
    }

    pub const fn identity(&self) -> &ApplicationCapabilityFieldBinding {
        &self.identity
    }

    pub fn context_relations(&self) -> &[ApplicationCapabilityRelationBinding] {
        &self.context_relations
    }
}

/// Application-owned projection of one complete proposed delegated grant.
///
/// Command authority remains outside this projection. Query independently
/// admits that authority, samples time, resolves the exact parent and grantee,
/// and proves narrowing before the proposed child may become active.
pub trait ApplicationCapabilityDelegationRequest<Schema, Operation> {
    type Scope;
    type Context;

    fn delegation_request(
        &self,
    ) -> Result<
        ApplicationCapabilityDelegationRequestProjection<Schema, Self::Scope, Self::Context>,
        ApplicationCapabilityDelegationRequestProjectionDenial,
    >;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationCapabilityDelegationRequestProjectionDenial {
    subject: String,
}

impl ApplicationCapabilityDelegationRequestProjectionDenial {
    pub fn input_variant(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
        }
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

/// Exact proposed child meaning without caller-authored grantor or status.
pub struct ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context> {
    target: ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
    parent: ErasedApplicationCapabilityEntitySelector,
    grantee: ErasedApplicationCapabilityEntitySelector,
    child_key: String,
    child_identity: ApplicationCapabilityValueBinding,
    workflow: ApplicationCapabilityValueBinding,
    not_before: ApplicationCapabilityValueBinding,
    not_after: ApplicationCapabilityValueBinding,
    remaining_delegations: ApplicationCapabilityValueBinding,
    activation_context: Vec<ApplicationCapabilityRelatedEntitySelector<Schema>>,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema, Scope, Context>
    ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new<Parent, Grantee>(
        target: ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
        parent: ApplicationCapabilityEntitySelector<Schema, Parent>,
        grantee: ApplicationCapabilityEntitySelector<Schema, Grantee>,
        child_key: impl Into<String>,
        child_identity: ApplicationCapabilityValueBinding,
        workflow: ApplicationCapabilityValueBinding,
        not_before: ApplicationCapabilityValueBinding,
        not_after: ApplicationCapabilityValueBinding,
        remaining_delegations: ApplicationCapabilityValueBinding,
        activation_context: impl IntoIterator<Item = ApplicationCapabilityRelatedEntitySelector<Schema>>,
    ) -> Result<Self, ApplicationCapabilityDelegationRequestProjectionDenial> {
        let child_key = child_key.into();
        if !valid_entity_key(&child_key) {
            return Err(
                ApplicationCapabilityDelegationRequestProjectionDenial::input_variant(
                    "delegated capability entity key",
                ),
            );
        }
        let activation_context = activation_context.into_iter().collect::<Vec<_>>();
        let distinct_context_relations = activation_context
            .iter()
            .map(|projection| projection.relation())
            .collect::<BTreeSet<_>>();
        if activation_context.len() > MAXIMUM_ACTIVATION_CONTEXT_RELATIONS
            || distinct_context_relations.len() != activation_context.len()
        {
            return Err(
                ApplicationCapabilityDelegationRequestProjectionDenial::input_variant(
                    "delegation activation context relations",
                ),
            );
        }
        Ok(Self {
            target,
            parent: parent.erase(),
            grantee: grantee.erase(),
            child_key,
            child_identity,
            workflow,
            not_before,
            not_after,
            remaining_delegations,
            activation_context,
            _schema: PhantomData,
        })
    }

    pub const fn target(&self) -> &ApplicationCapabilityRequestProjection<Schema, Scope, Context> {
        &self.target
    }

    pub const fn parent(&self) -> &ErasedApplicationCapabilityEntitySelector {
        &self.parent
    }

    pub const fn grantee(&self) -> &ErasedApplicationCapabilityEntitySelector {
        &self.grantee
    }

    pub fn child_key(&self) -> &str {
        &self.child_key
    }

    pub const fn child_identity(&self) -> &ApplicationCapabilityValueBinding {
        &self.child_identity
    }

    pub const fn workflow(&self) -> &ApplicationCapabilityValueBinding {
        &self.workflow
    }

    pub const fn not_before(&self) -> &ApplicationCapabilityValueBinding {
        &self.not_before
    }

    pub const fn not_after(&self) -> &ApplicationCapabilityValueBinding {
        &self.not_after
    }

    pub const fn remaining_delegations(&self) -> &ApplicationCapabilityValueBinding {
        &self.remaining_delegations
    }

    pub fn activation_context(&self) -> &[ApplicationCapabilityRelatedEntitySelector<Schema>] {
        &self.activation_context
    }
}

fn valid_entity_key(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.len() <= 512
        && !value.chars().any(char::is_control)
}
