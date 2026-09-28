use std::collections::{BTreeMap, HashSet};

use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use crate::physical_runtime::durability::{
    DataDispatchedPhysicalMutation, DataSettledPhysicalMutation, PageWalBasis,
};
use crate::physical_runtime::{
    record_serving::{CandidateFrameEffectSettlement, CandidateFrameEffectSource},
    PhysicalEffectIdentity, PhysicalWorkEffectFate, PhysicalWorkIdentity,
    PhysicalWorkRecoveryDisposition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDataEffectSource {
    NewArtifact,
    ExistingArtifactWriteback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalDataEffectSettlement {
    basis: PageWalBasis,
    source: PhysicalDataEffectSource,
    coordinate: RecordFrameCoordinate,
    payload_digest: [u8; 32],
    work: PhysicalWorkIdentity,
    effect: Option<PhysicalEffectIdentity>,
    fate: PhysicalWorkEffectFate,
    recovery: PhysicalWorkRecoveryDisposition,
}

pub enum PhysicalDataSettlementOutcome {
    Settled(DataSettledPhysicalMutation),
    InspectionRequired {
        dispatched: DataDispatchedPhysicalMutation,
        cause: PhysicalDataSettlementFailureCause,
    },
}

pub(in crate::physical_runtime) struct CompletionBoundPhysicalDataSettlement(
    DataDispatchedPhysicalMutation,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDataSettlementFailureCause {
    EmptyEffectSet,
    EffectCountMismatch,
    WalCoverageMismatch,
    BasisSubstitution,
    TargetSubstitution,
    PayloadSubstitution,
    EffectNotCompleted,
    EffectIdentityMissing,
    EffectSourceMismatch,
    DuplicateWorkIdentity,
}

impl PhysicalDataEffectSettlement {
    pub(in crate::physical_runtime) fn from_candidate(
        basis: PageWalBasis,
        effect: CandidateFrameEffectSettlement,
    ) -> Self {
        Self {
            basis,
            source: match effect.source() {
                CandidateFrameEffectSource::NewArtifact => PhysicalDataEffectSource::NewArtifact,
                CandidateFrameEffectSource::ExistingArtifactWriteback => {
                    PhysicalDataEffectSource::ExistingArtifactWriteback
                }
            },
            coordinate: effect.coordinate(),
            payload_digest: effect.payload_digest(),
            work: effect.work(),
            effect: effect.effect(),
            fate: effect.fate(),
            recovery: effect.recovery(),
        }
    }

    pub const fn basis(&self) -> &PageWalBasis {
        &self.basis
    }

    pub const fn source(&self) -> PhysicalDataEffectSource {
        self.source
    }

    pub const fn coordinate(&self) -> RecordFrameCoordinate {
        self.coordinate
    }

    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }

    pub const fn work_identity(&self) -> PhysicalWorkIdentity {
        self.work
    }

    pub const fn effect_identity(&self) -> Option<PhysicalEffectIdentity> {
        self.effect
    }

    pub const fn effect_fate(&self) -> PhysicalWorkEffectFate {
        self.fate
    }

    pub const fn recovery(&self) -> PhysicalWorkRecoveryDisposition {
        self.recovery
    }
}

pub(in crate::physical_runtime) fn join_dispatched_data(
    dispatched: DataDispatchedPhysicalMutation,
) -> PhysicalDataSettlementOutcome {
    let failure = validate(&dispatched).err();
    match failure {
        Some(cause) => PhysicalDataSettlementOutcome::InspectionRequired { dispatched, cause },
        None => PhysicalDataSettlementOutcome::Settled(DataSettledPhysicalMutation::new(
            CompletionBoundPhysicalDataSettlement(dispatched),
        )),
    }
}

impl CompletionBoundPhysicalDataSettlement {
    pub(in crate::physical_runtime) fn into_dispatched(self) -> DataDispatchedPhysicalMutation {
        self.0
    }
}

fn validate(
    dispatched: &DataDispatchedPhysicalMutation,
) -> Result<(), PhysicalDataSettlementFailureCause> {
    let Some(expected) = dispatched.durable().data_frames() else {
        let (copy, publication) = dispatched
            .durable()
            .source_copy()
            .ok_or(PhysicalDataSettlementFailureCause::BasisSubstitution)?;
        if !dispatched.effects().is_empty()
            || copy.require_live_source().is_err()
            || copy.writes().last_work().runtime()
                != dispatched.mutation_identity().runtime_identity()
            || copy.writes().frames() != copy.intent().chunk_count() + 1
            || !copy.synchronization().matches_writes(copy.writes())
            || publication != dispatched.durable().member_basis().lsn_range()
            || publication.start().get() <= copy.durable_intent_lsn()
        {
            return Err(PhysicalDataSettlementFailureCause::BasisSubstitution);
        }
        return Ok(());
    };
    let effects = dispatched.effects();
    if expected.is_empty() || effects.is_empty() {
        return Err(PhysicalDataSettlementFailureCause::EmptyEffectSet);
    }
    if expected.len() != effects.len() {
        return Err(PhysicalDataSettlementFailureCause::EffectCountMismatch);
    }
    let range = dispatched.durable().member_basis().lsn_range();
    let mut work = HashSet::new();
    let mut artifact_counts = BTreeMap::<RecordArtifactFile, usize>::new();
    for (frame, effect) in expected.iter().zip(effects) {
        if frame.basis() != effect.basis() {
            return Err(PhysicalDataSettlementFailureCause::BasisSubstitution);
        }
        if frame
            .basis()
            .delta()
            .iter()
            .any(|redo| !range.contains(redo.lsn()))
        {
            return Err(PhysicalDataSettlementFailureCause::WalCoverageMismatch);
        }
        let target = frame.basis().target().coordinate();
        if effect.coordinate() != target {
            return Err(PhysicalDataSettlementFailureCause::TargetSubstitution);
        }
        if effect.payload_digest() != frame.basis().resulting_payload_digest() {
            return Err(PhysicalDataSettlementFailureCause::PayloadSubstitution);
        }
        if !effect_has_completed_fate(effect)
            || effect.recovery() == PhysicalWorkRecoveryDisposition::InspectionRequired
        {
            return Err(PhysicalDataSettlementFailureCause::EffectNotCompleted);
        }
        if effect.effect_identity().is_none() {
            return Err(PhysicalDataSettlementFailureCause::EffectIdentityMissing);
        }
        if !work.insert(effect.work_identity()) {
            return Err(PhysicalDataSettlementFailureCause::DuplicateWorkIdentity);
        }
        let count = artifact_counts.entry(target.artifact()).or_default();
        let expected_source = if matches!(
            target.artifact(),
            worth_store_physical_format::RecordArtifactFile::ExtentArena { .. }
        ) || (*count == 0 && target.offset() == 0)
        {
            PhysicalDataEffectSource::NewArtifact
        } else {
            PhysicalDataEffectSource::ExistingArtifactWriteback
        };
        if effect.source() != expected_source {
            return Err(PhysicalDataSettlementFailureCause::EffectSourceMismatch);
        }
        *count += 1;
    }
    Ok(())
}

fn effect_has_completed_fate(effect: &PhysicalDataEffectSettlement) -> bool {
    matches!(
        (effect.source(), effect.effect_fate()),
        (
            PhysicalDataEffectSource::NewArtifact,
            PhysicalWorkEffectFate::PublicationCompleted
        ) | (
            PhysicalDataEffectSource::ExistingArtifactWriteback,
            PhysicalWorkEffectFate::WriteCompleted
        )
    )
}
