use std::any::TypeId;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use worth_relational::facade::identity::EntityId;

pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;

mod candidate;
pub(in crate::domain_computation::primary_graph::application_attempt) use candidate::WorthQueryApplicationOutputCorrespondenceCandidate;

#[cfg(test)]
mod tests;

pub(super) mod action {
    pub trait Sealed {
        const POSTURE: super::WorthQueryApplicationOutputPosture;
    }
}

pub struct Preserve;
pub struct Create;
pub struct Retire;

impl action::Sealed for Preserve {
    const POSTURE: WorthQueryApplicationOutputPosture =
        WorthQueryApplicationOutputPosture::Preserve;
}

impl action::Sealed for Create {
    const POSTURE: WorthQueryApplicationOutputPosture = WorthQueryApplicationOutputPosture::Create;
}

impl action::Sealed for Retire {
    const POSTURE: WorthQueryApplicationOutputPosture = WorthQueryApplicationOutputPosture::Retire;
}

/// A binding-owned semantic result role. The name describes correspondence;
/// it never supplies or reconstructs an entity identity.
pub struct WorthQueryApplicationOutputRole<Binding, Entity, Action> {
    name: &'static str,
    _marker: PhantomData<fn() -> (Binding, Entity, Action)>,
}

impl<Binding, Entity, Action> Copy for WorthQueryApplicationOutputRole<Binding, Entity, Action> {}

impl<Binding, Entity, Action> Clone for WorthQueryApplicationOutputRole<Binding, Entity, Action> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Binding, Entity, Action> WorthQueryApplicationOutputRole<Binding, Entity, Action> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }
}

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

impl WorthQueryApplicationOutputCorrespondence {
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
        Action: action::Sealed,
    {
        if self.binding_type != Some(TypeId::of::<Binding>()) {
            return Err(WorthQueryApplicationOutputProjectionDenial::ForeignBinding);
        }
        let binding = self
            .roles
            .get(role.name)
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
