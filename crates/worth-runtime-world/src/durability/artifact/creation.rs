//! The original creation binds its source, destination and actual owner effects.
use super::{DurableWorldHead, WorldSignalEvidence};
use crate::branch::SignalBranchCreationPlan;
use crate::durability::RecordedWorldInitialComponents;
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableProductUnpublishedOwnerEffectsIdentity, HomeIncarnation,
};
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence, history::BranchId, publication::PatchStreamPosition,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRecordedRelationalCreationPlan {
    ReuseExact,
    ForkExact {
        target: BranchId,
        canonical_before: PatchStreamPosition,
        transition_before: RelationalBranchTransitionSequence,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldCreationIntentRecord {
    pub(crate) association: CompositeAttemptAssociation,
    pub(crate) custody: DurableProductUnpublishedOwnerEffectsIdentity,
    pub(crate) source: DurableWorldHead,
    pub(crate) destination: DurableWorldHead,
    pub(crate) program_revision: [u8; 32],
    pub(crate) relational: WorldRecordedRelationalCreationPlan,
    pub(crate) signal: SignalBranchCreationPlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldRelationalForkEvidence {
    pub(crate) sequence: RelationalBranchTransitionSequence,
    pub(crate) digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldCreatedBranchOwnerEvidence {
    Reused,
    Relational(WorldRelationalForkEvidence),
    Signal(WorldSignalEvidence),
    RelationalAndSignal {
        relational: WorldRelationalForkEvidence,
        signal: WorldSignalEvidence,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldBranchCreatedRecord {
    pub(crate) intent: WorldCreationIntentRecord,
    pub(crate) components: RecordedWorldInitialComponents,
    pub(crate) owners: WorldCreatedBranchOwnerEvidence,
    pub(crate) publishing_incarnation: HomeIncarnation,
}

impl WorldCreationIntentRecord {
    pub(in crate::durability) fn validate_description(
        &self,
    ) -> Result<(), crate::durability::WorldDurableFrameDenial> {
        use crate::durability::WorldDurableFrameDenial as Frame;
        use SignalBranchCreationPlan as Signal;
        use WorldRecordedRelationalCreationPlan as Native;
        let intent = self;
        let owner = intent.association.attempt().owner();
        let native_forks = match intent.relational {
            Native::ReuseExact => false,
            Native::ForkExact { .. } => true,
        };
        let signal_forks = match intent.signal {
            Signal::ReuseExact => false,
            Signal::ForkExact { .. } => true,
        };
        if intent.association.branch() != &intent.source.branch
            || intent.association.incarnation() != intent.source.incarnation
            || intent.custody.owner() != owner
            || intent.source.branch.owner() != owner
            || intent.destination.incarnation.owner() != owner
            || intent.destination.branch.owner() != owner
            || intent.source.commit.owner() != owner
            || intent.destination.commit.owner() != owner
            || intent.source.branch == intent.destination.branch
            || intent.destination.incarnation.ordinal() <= intent.source.incarnation.ordinal()
            || intent.destination.generation.get() != 0
            || ((native_forks || signal_forks)
                == (intent.destination.commit == intent.source.commit))
        {
            return Err(Frame::IdentityMismatch);
        }
        Ok(())
    }
}

impl WorldBranchCreatedRecord {
    pub(in crate::durability) fn validate_description(
        &self,
    ) -> Result<(), crate::durability::WorldDurableFrameDenial> {
        use WorldCreatedBranchOwnerEvidence as Owners;
        use WorldRecordedRelationalCreationPlan as Native;
        self.intent.validate_description()?;
        let requested_native = match self.intent.relational {
            Native::ReuseExact => false,
            Native::ForkExact { .. } => true,
        };
        let requested_signal = match self.intent.signal {
            SignalBranchCreationPlan::ReuseExact => false,
            SignalBranchCreationPlan::ForkExact { .. } => true,
        };
        let (native, signal) = match &self.owners {
            Owners::Reused => (None, None),
            Owners::Relational(native) => (Some(native), None),
            Owners::Signal(signal) => (None, Some(signal)),
            Owners::RelationalAndSignal { relational, signal } => (Some(relational), Some(signal)),
        };
        if (requested_native, requested_signal) != (native.is_some(), signal.is_some())
            || native.is_some_and(|fork| fork.sequence.0 == 0)
            || signal.is_some_and(|point| point != &self.components.signal)
        {
            return Err(crate::durability::WorldDurableFrameDenial::IdentityMismatch);
        }
        Ok(())
    }
}
