//! "Explain authority": everything one principal may do on one scope, read
//! from the installed abilities and capabilities themselves.
//!
//! Every installed ability on the scope entity is decided by the policy
//! evaluator admission uses; every installed capability on it whose own
//! target is a complete request runs admission's capability observation.
//! An inventory reveals another principal's authority, so it is disclosed
//! only to an asker holding the disclosure ability on the scope at the head.
//! Nothing is retained, reserved or committed, and an entry is not authority.

mod observation;

use std::marker::PhantomData;

use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationAbilityRef, ApplicationAbilityVocabulary, ApplicationBindingContext,
    ApplicationRoleMatrix, ApplicationSchema,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;

use super::{WorthQueryAbilityEligibility, WorthQueryAbilityKey};
use crate::domain_computation::authorization::WorthQueryApplicationAuthorizationExplanationCause;

/// Whose authority on which scope an inventory explains, and the ability the
/// asker must hold on that scope to see it.
pub struct WorthQueryAuthorityQuestion<Schema, Context, Field: RecordSelection> {
    subject: WorthQueryExternalPrincipalIdentity,
    field: Field,
    value: Field::Value,
    disclosure: WorthQueryAbilityKey,
    _context: PhantomData<fn() -> (Schema, Context)>,
}

impl<Schema, Context, Field> WorthQueryAuthorityQuestion<Schema, Context, Field>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
    Field: RecordSelection,
{
    pub fn new<Disclosure>(
        _context: Context,
        subject: WorthQueryExternalPrincipalIdentity,
        field: Field,
        value: Field::Value,
        disclosure: ApplicationAbilityRef<Schema, Disclosure, Field::Entity>,
    ) -> Self {
        Self {
            subject,
            field,
            value,
            disclosure: WorthQueryAbilityKey::of(disclosure),
            _context: PhantomData,
        }
    }

    pub fn subject(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.subject
    }

    pub const fn disclosure(&self) -> WorthQueryAbilityKey {
        self.disclosure
    }

    pub(crate) fn scope_selection(&self) -> (Field, Field::Value)
    where
        Field::Value: Clone,
    {
        (self.field, self.value.clone())
    }
}

/// One installed authority on the scope entity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryAuthority {
    /// An ability and the installed policy deciding it.
    Ability { ability: String, policy: String },
    /// A capability and the operation it is exercised through.
    Capability {
        capability: String,
        operation: String,
    },
}

/// One relationship path, as the relations it crosses in order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthorityPath {
    pub relations: Vec<String>,
}

/// Whether admission would allow one authority, and why.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryAuthorityVerdict {
    /// Allowed through these paths: the ability policy's allowing paths that
    /// held, or the capability's grant path (grantee, then resource).
    Allowed {
        through: Vec<WorthQueryAuthorityPath>,
    },
    /// Refused. A capability names its policy cause; an ability policy has
    /// no finer cause than "no allowing path held, or a denying one did".
    Denied(Option<WorthQueryApplicationAuthorizationExplanationCause>),
    /// The capability's target leaves an elevation, related entity, field,
    /// magnitude or context entity to its request: ask it with a
    /// `WorthQueryCapabilityEligibilityQuestion` carrying that request.
    RequiresRequest,
}

impl WorthQueryAuthorityVerdict {
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }
}

/// One authority with its verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthorityEntry {
    pub authority: WorthQueryAuthority,
    pub verdict: WorthQueryAuthorityVerdict,
}

/// Everything the subject may do on the scope, abilities first, each kind
/// in installed order. A subject that is not one admitted principal has no
/// entries and says why.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthorityInventory {
    standing: WorthQueryAbilityEligibility,
    scope_entity: &'static str,
    entries: Vec<WorthQueryAuthorityEntry>,
}

impl WorthQueryAuthorityInventory {
    pub(crate) fn new(
        standing: WorthQueryAbilityEligibility,
        scope_entity: &'static str,
        entries: Vec<WorthQueryAuthorityEntry>,
    ) -> Self {
        Self {
            standing,
            scope_entity,
            entries,
        }
    }

    /// `Eligible` when the subject resolved to one principal; otherwise
    /// `Unmapped`, `Disabled` or `Ambiguous`, and the inventory is empty.
    pub const fn subject_standing(&self) -> WorthQueryAbilityEligibility {
        self.standing
    }

    pub const fn scope_entity(&self) -> &'static str {
        self.scope_entity
    }

    pub fn entries(&self) -> &[WorthQueryAuthorityEntry] {
        &self.entries
    }

    /// The authorities admission would allow.
    pub fn allowed(&self) -> impl Iterator<Item = &WorthQueryAuthority> {
        self.entries
            .iter()
            .filter(|entry| entry.verdict.is_allowed())
            .map(|entry| &entry.authority)
    }

    /// The verdict for `authority`, when installed on this scope.
    pub fn verdict(&self, authority: &WorthQueryAuthority) -> Option<&WorthQueryAuthorityVerdict> {
        self.entries
            .iter()
            .find(|entry| &entry.authority == authority)
            .map(|entry| &entry.verdict)
    }
}

/// What a role grants, from the schema's role matrix: the static half of an
/// authority explanation, needing no principal or basis.
pub fn role_authority<Schema, Role, Ability>(role: Role) -> Vec<WorthQueryAbilityKey>
where
    Role: ApplicationRoleMatrix<Ability>,
    Ability: ApplicationAbilityVocabulary<Schema>,
{
    role.abilities()
        .iter()
        .map(|ability| WorthQueryAbilityKey::named(ability.ability_name(), Ability::scope_name()))
        .collect()
}
