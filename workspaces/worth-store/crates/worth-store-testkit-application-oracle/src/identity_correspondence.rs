//! One run's identity relationships, retained across every Home reopen.
use std::collections::HashMap;
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_relational::facade::{
    history::{BranchId, CommitId},
    identity::{EntityId, LineageId, RelationId, VersionId},
};

/// Only objects observed in the compared durable prefix enter this vocabulary.
/// Reference coordinates remain memory coordinates; these values grant nothing.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ObservedDurableIdentity {
    Entity(EntityId),
    Relation(RelationId),
    Lineage(LineageId),
    NativeCommit(CommitId),
    NativeVersion(VersionId),
    NativeBranch(BranchId),
    WorldOwner(WorldOwnerCoordinate),
    WorldBranch {
        owner: WorldOwnerCoordinate,
        name: String,
    },
    WorldIncarnation {
        owner: WorldOwnerCoordinate,
        ordinal: u64,
    },
    WorldCommit {
        owner: WorldOwnerCoordinate,
        ordinal: u64,
    },
    WorldBootstrap {
        owner: WorldOwnerCoordinate,
        ordinal: u64,
    },
    WorldAttempt {
        owner: WorldOwnerCoordinate,
        ordinal: u64,
    },
    WorldCustody {
        owner: WorldOwnerCoordinate,
        ordinal: u64,
    },
    QueryOutcome(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityFamily {
    Entity,
    Relation,
    Lineage,
    NativeCommit,
    NativeVersion,
    NativeBranch,
    WorldOwner,
    WorldBranch,
    WorldIncarnation,
    WorldCommit,
    WorldBootstrap,
    WorldAttempt,
    WorldCustody,
    QueryOutcome,
}
impl ObservedDurableIdentity {
    pub fn family(&self) -> IdentityFamily {
        match self {
            Self::Entity(_) => IdentityFamily::Entity,
            Self::Relation(_) => IdentityFamily::Relation,
            Self::Lineage(_) => IdentityFamily::Lineage,
            Self::NativeCommit(_) => IdentityFamily::NativeCommit,
            Self::NativeVersion(_) => IdentityFamily::NativeVersion,
            Self::NativeBranch(_) => IdentityFamily::NativeBranch,
            Self::WorldOwner(_) => IdentityFamily::WorldOwner,
            Self::WorldBranch { .. } => IdentityFamily::WorldBranch,
            Self::WorldIncarnation { .. } => IdentityFamily::WorldIncarnation,
            Self::WorldCommit { .. } => IdentityFamily::WorldCommit,
            Self::WorldBootstrap { .. } => IdentityFamily::WorldBootstrap,
            Self::WorldAttempt { .. } => IdentityFamily::WorldAttempt,
            Self::WorldCustody { .. } => IdentityFamily::WorldCustody,
            Self::QueryOutcome(_) => IdentityFamily::QueryOutcome,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityDivergence {
    Family {
        home: IdentityFamily,
        reference: IdentityFamily,
    },
    HomeRemapped {
        home: ObservedDurableIdentity,
        prior: ObservedDurableIdentity,
        proposed: ObservedDurableIdentity,
    },
    ReferenceCollision {
        reference: ObservedDurableIdentity,
        prior: ObservedDurableIdentity,
        proposed: ObservedDurableIdentity,
    },
}

#[derive(Default)]
pub struct DurableIdentityCorrespondence {
    home_to_reference: HashMap<ObservedDurableIdentity, ObservedDurableIdentity>,
    reference_to_home: HashMap<ObservedDurableIdentity, ObservedDurableIdentity>,
}
impl DurableIdentityCorrespondence {
    /// Both directions are checked before either map changes. Reobserving a
    /// durable object after reopen must admit the same original relationship.
    pub fn admit(
        &mut self,
        home: ObservedDurableIdentity,
        reference: ObservedDurableIdentity,
    ) -> Result<(), IdentityDivergence> {
        if home.family() != reference.family() {
            return Err(IdentityDivergence::Family {
                home: home.family(),
                reference: reference.family(),
            });
        }
        if let Some(prior) = self.home_to_reference.get(&home) {
            if prior != &reference {
                return Err(IdentityDivergence::HomeRemapped {
                    home,
                    prior: prior.clone(),
                    proposed: reference,
                });
            }
        }
        if let Some(prior) = self.reference_to_home.get(&reference) {
            if prior != &home {
                return Err(IdentityDivergence::ReferenceCollision {
                    reference,
                    prior: prior.clone(),
                    proposed: home,
                });
            }
        }
        self.home_to_reference
            .insert(home.clone(), reference.clone());
        self.reference_to_home.insert(reference, home);
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.home_to_reference.len()
    }
    pub fn is_empty(&self) -> bool {
        self.home_to_reference.is_empty()
    }
}
