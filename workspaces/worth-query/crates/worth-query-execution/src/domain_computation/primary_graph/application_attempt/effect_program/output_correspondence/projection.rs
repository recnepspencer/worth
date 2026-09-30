use std::marker::PhantomData;

use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRoleNameDenial;
use worth_relational::facade::identity::EntityId;

/// The committed entity behind one declared output role, projected from a
/// `WorthQueryApplicationOutputCorrespondence` with its contract, entity type
/// and posture checked.
///
/// Get one from the correspondence's `entity` or `member`; read the identity
/// with `entity_id`. It describes what the commit produced and grants nothing.
pub struct WorthQueryApplicationOutputEntity<Entity, Action> {
    entity_id: EntityId,
    _marker: PhantomData<fn() -> (Entity, Action)>,
}

impl<Entity, Action> WorthQueryApplicationOutputEntity<Entity, Action> {
    pub(super) const fn new(entity_id: EntityId) -> Self {
        Self {
            entity_id,
            _marker: PhantomData,
        }
    }

    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

/// Why an output role could not be projected from a committed output
/// correspondence. Nothing is changed by the refusal.
///
/// A role marker its contract does not declare exactly as used never reaches
/// here: the use fails to compile. These refusals cover what the type system
/// cannot see, such as a correspondence committed under another contract or
/// readmitted from a checkpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationOutputProjectionDenial {
    /// The commit bound no entity to the requested exactly-one role.
    MissingRole,
    /// The role's cardinality differs from the one the committing binding
    /// declared.
    CardinalityMismatch,
    /// The correspondence was committed by a different mutation binding than
    /// the one requested.
    ForeignBinding,
    /// The correspondence was committed under a different output contract than
    /// the role marker's.
    ForeignContract,
    /// The family member suffix does not form a valid role name.
    InvalidMemberSuffix(WorthQueryApplicationOutputRoleNameDenial),
    /// The role was committed with a different posture (preserve, create, or
    /// retire) than the one requested.
    ActionMismatch,
    /// The role was committed for a different entity type than the one requested.
    EntityMismatch,
}
