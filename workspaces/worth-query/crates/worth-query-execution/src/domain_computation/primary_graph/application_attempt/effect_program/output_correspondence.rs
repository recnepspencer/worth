use std::any::TypeId;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use worth_relational::facade::identity::EntityId;

pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;

mod candidate;
pub(in crate::domain_computation::primary_graph::application_attempt) use candidate::WorthQueryApplicationOutputCorrespondenceCandidate;

mod role;
pub use role::{
    Create, Preserve, Retire, WorthQueryApplicationOutputAction, WorthQueryApplicationOutputRole,
    WorthQueryApplicationOutputRoleFamily, WorthQueryApplicationOutputRoleNameDenial,
};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommittedOutputBinding {
    posture: WorthQueryApplicationOutputPosture,
    entity_type: TypeId,
    entity: EntityId,
}

/// Sealed role-to-identity correspondence resolved from one Relational commit.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryApplicationOutputCorrespondence {
    binding_type: Option<TypeId>,
    roles: BTreeMap<String, CommittedOutputBinding>,
}

/// One typed member of a sealed output-role family.
pub struct WorthQueryApplicationOutputFamilyEntry<'correspondence, Binding, Entity> {
    role: &'correspondence str,
    posture: WorthQueryApplicationOutputPosture,
    entity_id: EntityId,
    _marker: PhantomData<fn() -> (Binding, Entity)>,
}

impl<Binding, Entity> Copy for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {}

impl<Binding, Entity> Clone for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Binding, Entity> std::fmt::Debug
    for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationOutputFamilyEntry")
            .field("role", &self.role)
            .field("posture", &self.posture)
            .field("entity_id", &self.entity_id)
            .finish()
    }
}

impl<Binding, Entity> PartialEq for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {
    fn eq(&self, other: &Self) -> bool {
        self.role == other.role
            && self.posture == other.posture
            && self.entity_id == other.entity_id
    }
}

impl<Binding, Entity> Eq for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {}

impl<Binding, Entity> WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {
    pub const fn role(&self) -> &str {
        self.role
    }

    pub const fn posture(&self) -> WorthQueryApplicationOutputPosture {
        self.posture
    }

    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

impl WorthQueryApplicationOutputCorrespondence {
    pub(in crate::domain_computation::primary_graph) const fn binding_type(
        &self,
    ) -> Option<TypeId> {
        self.binding_type
    }

    pub fn entity<Binding, Entity, Action>(
        &self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
    ) -> Result<
        WorthQueryApplicationOutputEntity<Binding, Entity, Action>,
        WorthQueryApplicationOutputProjectionDenial,
    >
    where
        Binding: 'static,
        Entity: 'static,
        Action: WorthQueryApplicationOutputAction,
    {
        if self.binding_type != Some(TypeId::of::<Binding>()) {
            return Err(WorthQueryApplicationOutputProjectionDenial::ForeignBinding);
        }
        let binding = self
            .roles
            .get(role.name())
            .ok_or(WorthQueryApplicationOutputProjectionDenial::MissingRole)?;
        if binding.posture != Action::POSTURE {
            return Err(WorthQueryApplicationOutputProjectionDenial::ActionMismatch);
        }
        if binding.entity_type != TypeId::of::<Entity>() {
            return Err(WorthQueryApplicationOutputProjectionDenial::EntityMismatch);
        }
        Ok(WorthQueryApplicationOutputEntity {
            entity_id: binding.entity,
            _marker: PhantomData,
        })
    }

    /// Projects every member matching one typed role-family prefix from this
    /// exact committed correspondence, preserving Query's authoritative posture.
    pub fn family_entries<Binding: 'static, Entity: 'static>(
        &self,
        family: WorthQueryApplicationOutputRoleFamily<Binding, Entity>,
    ) -> Result<
        Vec<WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity>>,
        WorthQueryApplicationOutputProjectionDenial,
    > {
        self.binding_family_entries::<Binding, Entity>(family.prefix())?
            .map(|entry| {
                entry.map(
                    |(role, posture, entity_id)| WorthQueryApplicationOutputFamilyEntry {
                        role,
                        posture,
                        entity_id,
                        _marker: PhantomData,
                    },
                )
            })
            .collect()
    }

    pub(in crate::domain_computation::primary_graph) fn entity_for_binding_role<
        Binding: 'static,
    >(
        &self,
        role: &str,
    ) -> Result<EntityId, WorthQueryApplicationOutputProjectionDenial> {
        if self.binding_type != Some(TypeId::of::<Binding>()) {
            return Err(WorthQueryApplicationOutputProjectionDenial::ForeignBinding);
        }
        self.roles
            .get(role)
            .map(|binding| binding.entity)
            .ok_or(WorthQueryApplicationOutputProjectionDenial::MissingRole)
    }

    pub(in crate::domain_computation::primary_graph) fn current_entity_for_role<Entity: 'static>(
        &self,
        role: &str,
    ) -> Result<Option<EntityId>, WorthQueryApplicationOutputProjectionDenial> {
        let Some(binding) = self.roles.get(role) else {
            return Ok(None);
        };
        if binding.entity_type != TypeId::of::<Entity>() {
            return Err(WorthQueryApplicationOutputProjectionDenial::EntityMismatch);
        }
        Ok(
            (binding.posture != WorthQueryApplicationOutputPosture::Retire)
                .then_some(binding.entity),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn binding_family_entries<
        'correspondence,
        Binding: 'static,
        Entity: 'static,
    >(
        &'correspondence self,
        prefix: &'correspondence str,
    ) -> Result<
        impl Iterator<
                Item = Result<
                    (
                        &'correspondence str,
                        WorthQueryApplicationOutputPosture,
                        EntityId,
                    ),
                    WorthQueryApplicationOutputProjectionDenial,
                >,
            > + 'correspondence,
        WorthQueryApplicationOutputProjectionDenial,
    > {
        use std::ops::Bound::{Excluded, Unbounded};

        if self.binding_type != Some(TypeId::of::<Binding>()) {
            return Err(WorthQueryApplicationOutputProjectionDenial::ForeignBinding);
        }
        Ok(self
            .roles
            .range::<str, _>((Excluded(prefix), Unbounded))
            .take_while(move |(role, _)| role.starts_with(prefix))
            .map(|(role, binding)| {
                if binding.entity_type != TypeId::of::<Entity>() {
                    return Err(WorthQueryApplicationOutputProjectionDenial::EntityMismatch);
                }
                Ok((role.as_str(), binding.posture, binding.entity))
            }))
    }
}

pub struct WorthQueryApplicationOutputEntity<Binding, Entity, Action> {
    entity_id: EntityId,
    _marker: PhantomData<fn() -> (Binding, Entity, Action)>,
}

impl<Binding, Entity, Action> WorthQueryApplicationOutputEntity<Binding, Entity, Action> {
    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationOutputProjectionDenial {
    MissingRole,
    ForeignBinding,
    ActionMismatch,
    EntityMismatch,
}
