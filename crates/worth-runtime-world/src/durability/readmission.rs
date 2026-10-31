//! Owner-frame readmission and cross-journal consistency. These are recovered
//! descriptions, never Native performed capabilities or live product heads.
use super::{
    DurableWorldHead, WorldDurableArtifact, WorldDurableHomeBinding, WorldDurableMediumDenial,
    WorldDurableRecoverySource, WorldIdentityFrontiers, WorldOpenAttemptRecord, WorldRecord,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableCompositeCommitIdentity,
    DurableCompositePublicationAttemptIdentity, DurableProductBranchIdentity,
    DurableProductUnpublishedOwnerEffectsIdentity, WorldDurableJournal,
};

#[derive(Debug)]
pub enum WorldReadmissionDenial {
    NativeCustody {
        cause: Box<WorldReadmissionDenial>,
        custody: Box<super::WorldNativeReadmissionCustody>,
    },
    NativeDiscovery(worth_relational::facade::durability::RelationalAssociatedEffectRefusal),
    NativeComponent(Box<worth_relational::facade::durability::RefusedRelationalRecordedComponent>),
    NativePublication(
        Box<worth_relational::facade::durability::RefusedRelationalPublicationReadmission>,
    ),
    NativeSuccessor(worth_relational::facade::durability::RelationalRecordedSuccessorRefusal),
    NativeSettlementRequired,
    OrphanNativeAssociation,
    DuplicateNativeEffect,
    MalformedNativeAssociation,
    DispositionEvidenceMismatch,
    ProgramRevisionMismatch,
    OccurrenceReadmissionRequired,
    RecoveryCapacity,
    ExecutionInstallation(crate::lifecycle::RuntimeWorldBuildDenial),
    Medium(WorldDurableMediumDenial),
    IdentityExhaustion(crate::identity::RuntimeWorldIdentityExhaustion),
    WrongJournal,
    ForeignOwner,
    DuplicateIdentity,
    MissingOpenAttempt,
    ConflictingOpenAttempt,
    ConflictingDisposition,
    MissingPredecessor,
    BranchLineage,
    FutureIncarnation,
    NativeReadmissionRequired,
    RootPreparation(Box<super::RefusedWorldRootReadmission>),
    SignalInitialRoot(worth_signal::facade::branch::SignalInitialEmptyRootRefusal),
    BridgeBaseline(worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal),
    NativeRecorded(worth_relational::facade::branch::RelationalRecordedComponentRefusal),
    RootAdmission(crate::branch::RuntimeWorldBootstrapNoEffectCause),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum RecoveredCustodyDisposition {
    Released(Option<super::WorldRelationalReleaseEvidence>),
    Continued,
}

#[derive(Debug)]
pub(crate) struct WorldRecoveredJournalState {
    pub(crate) frontiers: WorldIdentityFrontiers,
    pub(crate) branches: Vec<WorldDurableArtifact>,
    pub(crate) occurrences: Vec<WorldDurableArtifact>,
    pub(crate) opens: BTreeMap<DurableCompositePublicationAttemptIdentity, WorldOpenAttemptRecord>,
    pub(crate) dispositions:
        BTreeMap<DurableCompositePublicationAttemptIdentity, RecoveredCustodyDisposition>,
}

impl WorldRecoveredJournalState {
    pub(crate) fn readmit(
        source: WorldDurableRecoverySource,
        binding: &WorldDurableHomeBinding,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorldReadmissionDenial> {
        let mut recovered = Self {
            frontiers: WorldIdentityFrontiers::default(),
            branches: Vec::new(),
            occurrences: Vec::new(),
            opens: BTreeMap::new(),
            dispositions: BTreeMap::new(),
        };
        let mut heads = BTreeMap::new();
        let mut history = BTreeMap::new();
        let mut owner_recorded = false;
        for journal in [
            WorldDurableJournal::Branches,
            WorldDurableJournal::Custody,
            WorldDurableJournal::Occurrences,
        ] {
            for bytes in source.stream(journal) {
                execution.consult().map_err(|cause| {
                    WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Request(cause))
                })?;
                let artifact =
                    WorldDurableArtifact::readmit(bytes, binding.owner().home(), execution)
                        .map_err(WorldReadmissionDenial::Medium)?;
                if artifact.journal() != journal {
                    return Err(WorldReadmissionDenial::WrongJournal);
                }
                if artifact.record.owner() != binding.owner() {
                    return Err(WorldReadmissionDenial::ForeignOwner);
                }
                recovered
                    .frontiers
                    .include_record(&artifact.record)
                    .map_err(WorldReadmissionDenial::IdentityExhaustion)?;
                match &artifact.record {
                    WorldRecord::Owner { .. } => {
                        if owner_recorded {
                            return Err(WorldReadmissionDenial::DuplicateIdentity);
                        }
                        owner_recorded = true;
                    }
                    WorldRecord::Branch { head, .. } => {
                        if head.generation.get() != 0
                            || heads.insert(head.branch.clone(), head.clone()).is_some()
                            || history.insert(head.commit, head.clone()).is_some()
                        {
                            return Err(WorldReadmissionDenial::BranchLineage);
                        }
                        recovered.branches.push(artifact);
                    }
                    WorldRecord::OpenAttempt(open) => {
                        if recovered
                            .opens
                            .insert(open.association.attempt(), open.clone())
                            .is_some()
                        {
                            return Err(WorldReadmissionDenial::DuplicateIdentity);
                        }
                    }
                    WorldRecord::Released {
                        association,
                        custody,
                        relational,
                    } => {
                        recovered.readmit_disposition(
                            association,
                            *custody,
                            RecoveredCustodyDisposition::Released(relational.clone()),
                        )?;
                    }
                    WorldRecord::Continued {
                        association,
                        custody,
                    } => {
                        recovered.readmit_disposition(
                            association,
                            *custody,
                            RecoveredCustodyDisposition::Continued,
                        )?;
                    }
                    WorldRecord::Occurrence {
                        open,
                        publishing_incarnation,
                        ..
                    } => {
                        recovered.require_open(&open.association, open.custody)?;
                        if recovered.opens.get(&open.association.attempt()) != Some(open) {
                            return Err(WorldReadmissionDenial::ConflictingOpenAttempt);
                        }
                        if *publishing_incarnation > binding.incarnation() {
                            return Err(WorldReadmissionDenial::FutureIncarnation);
                        }
                        if matches!(
                            recovered.dispositions.get(&open.association.attempt()),
                            Some(RecoveredCustodyDisposition::Released(_))
                        ) {
                            return Err(WorldReadmissionDenial::ConflictingDisposition);
                        }
                        advance_head(&mut heads, &mut history, open)?;
                        recovered.occurrences.push(artifact);
                    }
                }
            }
        }
        recovered.validate_open_identities(&history)?;
        Ok(recovered)
    }

    fn require_open(
        &self,
        association: &CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
    ) -> Result<(), WorldReadmissionDenial> {
        let open = self
            .opens
            .get(&association.attempt())
            .ok_or(WorldReadmissionDenial::MissingOpenAttempt)?;
        if open.association != *association || open.custody != custody {
            return Err(WorldReadmissionDenial::ConflictingOpenAttempt);
        }
        Ok(())
    }

    fn readmit_disposition(
        &mut self,
        association: &CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
        disposition: RecoveredCustodyDisposition,
    ) -> Result<(), WorldReadmissionDenial> {
        self.require_open(association, custody)?;
        if self
            .dispositions
            .insert(association.attempt(), disposition)
            .is_some()
        {
            return Err(WorldReadmissionDenial::ConflictingDisposition);
        }
        Ok(())
    }

    fn validate_open_identities(
        &self,
        history: &BTreeMap<DurableCompositeCommitIdentity, DurableWorldHead>,
    ) -> Result<(), WorldReadmissionDenial> {
        let mut commits = BTreeSet::new();
        let mut custody = BTreeSet::new();
        for open in self.opens.values() {
            if !commits.insert(open.commit) || !custody.insert(open.custody) {
                return Err(WorldReadmissionDenial::DuplicateIdentity);
            }
            if history.get(&open.expected.commit) != Some(&open.expected) {
                return Err(WorldReadmissionDenial::MissingPredecessor);
            }
        }
        Ok(())
    }
}

fn advance_head(
    heads: &mut BTreeMap<DurableProductBranchIdentity, DurableWorldHead>,
    history: &mut BTreeMap<DurableCompositeCommitIdentity, DurableWorldHead>,
    open: &WorldOpenAttemptRecord,
) -> Result<(), WorldReadmissionDenial> {
    let current = heads
        .get_mut(&open.expected.branch)
        .ok_or(WorldReadmissionDenial::MissingPredecessor)?;
    if *current != open.expected {
        return Err(WorldReadmissionDenial::BranchLineage);
    }
    let generation = current
        .generation
        .get()
        .checked_add(1)
        .ok_or(WorldReadmissionDenial::BranchLineage)?;
    let successor = DurableWorldHead {
        branch: current.branch.clone(),
        incarnation: current.incarnation,
        generation: worth_foundational::facade::DurableProductReferenceGeneration::from_recorded(
            generation,
        ),
        commit: open.commit,
    };
    if history.insert(open.commit, successor.clone()).is_some() {
        return Err(WorldReadmissionDenial::DuplicateIdentity);
    }
    *current = successor;
    Ok(())
}
