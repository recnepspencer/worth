//! Fact-free eligibility observation: "who may act?" answered by the same
//! installed policy decision admission uses, without minting a decision fact,
//! reserving a grant or touching the ledger.
//!
//! A verdict describes the selected committed basis only, while the asker is
//! always admitted (and any roster disclosed) at the branch head. A verdict
//! is not authority: acting still admits the actor freshly.

mod capability;
mod observation;
mod prepared;

pub use capability::{
    WorthQueryCapabilityEligibility, WorthQueryCapabilityEligibilityQuestion,
    WorthQueryCapabilityKey,
};

use std::marker::PhantomData;

use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationAbilityRef, ApplicationBindingContext, ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenial;

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::resolution::WorthQueryCandidatePrincipalRefusal;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationReadObservation, WorthQueryEntityResolutionDenial,
    WorthQueryPrincipalResolutionDenial,
};

/// Whether one principal may exercise an ability on one scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAbilityEligibility {
    /// The installed policy allows it: admission would grant this ability.
    Eligible,
    /// The installed policy refuses it: no allowing path holds, or a
    /// prohibiting path does.
    Denied,
    /// No mapping resolves this identity to a principal.
    Unmapped,
    /// The identity's mapping is disabled.
    Disabled,
    /// The identity resolves to more than one principal.
    Ambiguous,
}

impl WorthQueryAbilityEligibility {
    pub const fn is_eligible(self) -> bool {
        matches!(self, Self::Eligible)
    }

    pub(crate) const fn of_policy(allowed: bool) -> Self {
        if allowed {
            Self::Eligible
        } else {
            Self::Denied
        }
    }

    pub(crate) const fn of_refusal(refusal: WorthQueryCandidatePrincipalRefusal) -> Self {
        match refusal {
            WorthQueryCandidatePrincipalRefusal::Unmapped => Self::Unmapped,
            WorthQueryCandidatePrincipalRefusal::Disabled => Self::Disabled,
            WorthQueryCandidatePrincipalRefusal::Ambiguous => Self::Ambiguous,
        }
    }
}

/// One installed ability on its scope entity, by schema identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryAbilityKey {
    ability: &'static str,
    scope_entity: &'static str,
}

impl WorthQueryAbilityKey {
    pub const fn of<Schema, Ability, Scope>(
        ability: ApplicationAbilityRef<Schema, Ability, Scope>,
    ) -> Self {
        Self {
            ability: ability.name(),
            scope_entity: ability.scope(),
        }
    }

    pub const fn ability(&self) -> &'static str {
        self.ability
    }

    pub const fn scope_entity(&self) -> &'static str {
        self.scope_entity
    }
}

/// One ability on one scope (`field == value`), asked through the principal
/// binding a `worth_query_principal_binding!` context names. The ability and
/// the scope field share one scope entity at compile time.
pub struct WorthQueryEligibilityQuestion<Schema, Context, Ability, Field: RecordSelection> {
    ability: ApplicationAbilityRef<Schema, Ability, Field::Entity>,
    field: Field,
    value: Field::Value,
    _context: PhantomData<fn() -> Context>,
}

impl<Schema, Context, Ability, Field> WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
    Field: RecordSelection,
{
    pub fn new(
        _context: Context,
        ability: ApplicationAbilityRef<Schema, Ability, Field::Entity>,
        field: Field,
        value: Field::Value,
    ) -> Self {
        Self {
            ability,
            field,
            value,
            _context: PhantomData,
        }
    }

    /// The installed ability this question asks about.
    pub const fn ability(&self) -> WorthQueryAbilityKey {
        WorthQueryAbilityKey::of(self.ability)
    }

    pub(crate) fn scope_selection(&self) -> (Field, Field::Value)
    where
        Field::Value: Clone,
    {
        (self.field, self.value.clone())
    }
}

/// The committed basis an observation reads.
#[derive(Clone, Copy)]
pub enum WorthQueryEligibilityBasis<'a> {
    /// The branch's current head, as an action admitted now would read it.
    Current(crate::basis::WorthQueryProductBranch),
    /// A retained read observation, as of its pinned commit.
    Retained(&'a WorthQueryApplicationReadObservation),
}

/// Why an eligibility question could not be answered. A refused question
/// mints nothing and leaves no effect.
#[derive(Debug)]
pub enum WorthQueryAbilityEligibilityDenial {
    /// The basis could not be selected.
    Basis(crate::basis::WorthQueryProductBranchAdmissionDenial),
    /// The principal binding is not installed in this runtime.
    PrincipalBinding(WorthQueryPrincipalBindingInstallationDenial),
    /// The ability has no installed policy on its scope entity.
    AbilityNotInstalled { ability: &'static str },
    /// The policy's principal entity is not the binding's principal entity.
    PrincipalEntityMismatch { ability: &'static str },
    /// The scope did not resolve at the selected basis.
    Scope(WorthQueryEntityResolutionDenial),
    /// The asking principal is not admitted, so it may not ask about others.
    Asker(WorthQueryPrincipalResolutionDenial),
    /// Resolving a candidate stopped for a reason other than its own mapping.
    Candidate(WorthQueryPrincipalResolutionDenial),
    /// The asker does not hold the roster's disclosure ability on this scope.
    RosterNotDisclosed { disclosure: &'static str },
    /// The policy decision was refused or stopped before a verdict.
    Authorization(WorthQueryOperationAuthorizationDenial),
    /// The capability is not installed for its operation in this schema.
    CapabilityNotInstalled { capability: &'static str },
    /// The capability is exercised only through an approved elevation, which
    /// a fact-free question cannot carry.
    CapabilityRequiresElevation { capability: &'static str },
    /// The capability request did not resolve at the selected basis.
    CapabilityRequest(WorthQueryOperationAuthorizationDenial),
}
