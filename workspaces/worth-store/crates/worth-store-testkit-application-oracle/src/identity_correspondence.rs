//! One run's identity relationships, retained across every Home reopen.
use std::collections::HashMap;
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_relational::facade::{
    history::{BranchId, CommitId},
    identity::{EntityId, LineageId, PartitionId, RelationId, VersionId},
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
    NativePartition(PartitionId),
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
    NativePartition,
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
            Self::NativePartition(_) => IdentityFamily::NativePartition,
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
        self.check_pair(&home, &reference)?;
        let partitions = record_partitions(&home, &reference);
        if let Some((h, r)) = &partitions {
            self.check_pair(h, r)?;
        }
        if let Some((h, r)) = partitions {
            self.insert_pair(h, r);
        }
        self.insert_pair(home, reference);
        Ok(())
    }
    fn check_pair(
        &self,
        home: &ObservedDurableIdentity,
        reference: &ObservedDurableIdentity,
    ) -> Result<(), IdentityDivergence> {
        if home.family() != reference.family() {
            return Err(IdentityDivergence::Family {
                home: home.family(),
                reference: reference.family(),
            });
        }
        if let Some(prior) = self.home_to_reference.get(home)
            && prior != reference
        {
            return Err(IdentityDivergence::HomeRemapped {
                home: home.clone(),
                prior: prior.clone(),
                proposed: reference.clone(),
            });
        }
        if let Some(prior) = self.reference_to_home.get(reference)
            && prior != home
        {
            return Err(IdentityDivergence::ReferenceCollision {
                reference: reference.clone(),
                prior: prior.clone(),
                proposed: home.clone(),
            });
        }
        Ok(())
    }
    fn insert_pair(&mut self, home: ObservedDurableIdentity, reference: ObservedDurableIdentity) {
        self.home_to_reference
            .insert(home.clone(), reference.clone());
        self.reference_to_home.insert(reference, home);
    }
    pub fn len(&self) -> usize {
        self.home_to_reference.len()
    }
    pub fn is_empty(&self) -> bool {
        self.home_to_reference.is_empty()
    }
}

fn record_partitions(
    h: &ObservedDurableIdentity,
    r: &ObservedDurableIdentity,
) -> Option<(ObservedDurableIdentity, ObservedDurableIdentity)> {
    use ObservedDurableIdentity as I;
    let (h, r) = match h {
        I::Entity(h) => {
            let I::Entity(r) = r else {
                unreachable!("admission checked the same identity family");
            };
            (h.partition_id, r.partition_id)
        }
        I::Relation(h) => {
            let I::Relation(r) = r else {
                unreachable!("admission checked the same identity family");
            };
            (h.partition_id, r.partition_id)
        }
        I::Lineage(_)
        | I::NativeCommit(_)
        | I::NativeVersion(_)
        | I::NativeBranch(_)
        | I::NativePartition(_)
        | I::WorldOwner(_)
        | I::WorldBranch { .. }
        | I::WorldIncarnation { .. }
        | I::WorldCommit { .. }
        | I::WorldBootstrap { .. }
        | I::WorldAttempt { .. }
        | I::WorldCustody { .. }
        | I::QueryOutcome(_) => return None,
    };
    Some((I::NativePartition(h), I::NativePartition(r)))
}
