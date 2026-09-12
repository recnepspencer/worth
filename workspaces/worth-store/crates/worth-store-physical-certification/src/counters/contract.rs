use std::collections::BTreeSet;

use super::{
    classify_counter_strength, counter_expectation_kind_token, CounterExpectationStrength,
    CounterStrengthJustification, CounterStrengthPosture, OverExactCounterDenied,
    PhysicalCounterExpectation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CounterContractKind {
    ActorStepExact,
    ForbiddenShortcutExact,
    ReplayIdentityExact,
    ProfileResourceEnvelope,
    BlobChunkCountExact,
    BlobLogicalBytesExact,
    AllocationBytes,
    PagePins,
    IoQueueDepth,
    ResidentBytes,
    DirtyPages,
    IoInterferenceEvents,
    LatchWaits,
    EpochRetries,
    ProtectedReferences,
    Retries,
    BlockedReclaimAttempts,
    CompactionPublicationPlanCompletions,
    ReplayedPages,
    CompactionCandidateRanges,
    CopiedPages,
    FutureS5SpecificCounters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalCounterContract {
    kind: CounterContractKind,
    expectation: PhysicalCounterExpectation,
    posture: CounterStrengthPosture,
    justification: CounterStrengthJustification,
}

impl PhysicalCounterContract {
    pub(crate) fn exact(kind: CounterContractKind, expected: u64) -> Self {
        Self::try_new(kind, PhysicalCounterExpectation::exact(expected))
            .expect("exact contract constructors only target exact counter claims")
    }

    pub(crate) fn profile_scoped(kind: CounterContractKind) -> Self {
        Self::try_new(kind, PhysicalCounterExpectation::profile_scoped())
            .expect("profile-scoped contract constructor is statically valid")
    }

    pub fn try_new(
        kind: CounterContractKind,
        expectation: PhysicalCounterExpectation,
    ) -> Result<Self, OverExactCounterDenied> {
        let (posture, justification) = classify_counter_strength(kind, expectation.kind())?;
        Ok(Self {
            kind,
            expectation,
            posture,
            justification,
        })
    }

    pub fn zero(kind: CounterContractKind) -> Result<Self, OverExactCounterDenied> {
        Self::try_new(kind, PhysicalCounterExpectation::zero())
    }

    pub fn positive(kind: CounterContractKind) -> Result<Self, OverExactCounterDenied> {
        Self::try_new(kind, PhysicalCounterExpectation::positive())
    }

    pub fn monotonic(kind: CounterContractKind) -> Result<Self, OverExactCounterDenied> {
        Self::try_new(kind, PhysicalCounterExpectation::monotonic())
    }

    pub fn bounded(kind: CounterContractKind, maximum: u64) -> Result<Self, CounterContractDenial> {
        Self::try_new(
            kind,
            PhysicalCounterExpectation::bounded(maximum)
                .map_err(CounterContractDenial::InvalidExpectation)?,
        )
        .map_err(CounterContractDenial::OverExact)
    }

    pub const fn kind(&self) -> CounterContractKind {
        self.kind
    }

    pub const fn expectation(&self) -> &PhysicalCounterExpectation {
        &self.expectation
    }

    pub const fn strength(&self) -> CounterExpectationStrength {
        self.expectation.kind()
    }

    pub const fn posture(&self) -> CounterStrengthPosture {
        self.posture
    }

    pub const fn justification(&self) -> CounterStrengthJustification {
        self.justification
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterContractDenial {
    InvalidExpectation(super::CounterExpectationDenial),
    OverExact(OverExactCounterDenied),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredCounterContractSet {
    contracts: Vec<PhysicalCounterContract>,
}

impl RequiredCounterContractSet {
    pub(crate) fn from_contracts(
        contracts: impl IntoIterator<Item = PhysicalCounterContract>,
    ) -> Self {
        Self {
            contracts: contracts
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        }
    }

    pub fn contains(&self, kind: CounterContractKind) -> bool {
        self.contracts.iter().any(|contract| contract.kind == kind)
    }

    pub fn require(&self, kind: CounterContractKind) -> Option<&PhysicalCounterContract> {
        self.contracts.iter().find(|contract| contract.kind == kind)
    }

    pub fn iter(&self) -> impl Iterator<Item = &PhysicalCounterContract> {
        self.contracts.iter()
    }
}

pub(crate) fn counter_contract_kind_token(kind: CounterContractKind) -> &'static str {
    match kind {
        CounterContractKind::ActorStepExact => "actor-step-exact",
        CounterContractKind::ForbiddenShortcutExact => "forbidden-shortcut-exact",
        CounterContractKind::ReplayIdentityExact => "replay-identity-exact",
        CounterContractKind::ProfileResourceEnvelope => "profile-resource-envelope",
        CounterContractKind::BlobChunkCountExact => "blob-chunk-count-exact",
        CounterContractKind::BlobLogicalBytesExact => "blob-logical-bytes-exact",
        CounterContractKind::AllocationBytes => "allocation-bytes",
        CounterContractKind::PagePins => "page-pins",
        CounterContractKind::IoQueueDepth => "io-queue-depth",
        CounterContractKind::ResidentBytes => "resident-bytes",
        CounterContractKind::DirtyPages => "dirty-pages",
        CounterContractKind::IoInterferenceEvents => "io-interference-events",
        CounterContractKind::LatchWaits => "latch-waits",
        CounterContractKind::EpochRetries => "epoch-retries",
        CounterContractKind::ProtectedReferences => "protected-references",
        CounterContractKind::Retries => "retries",
        CounterContractKind::BlockedReclaimAttempts => "blocked-reclaim-attempts",
        CounterContractKind::CompactionPublicationPlanCompletions => {
            "compaction-publication-plan-completions"
        }
        CounterContractKind::ReplayedPages => "replayed-pages",
        CounterContractKind::CompactionCandidateRanges => "compaction-candidate-ranges",
        CounterContractKind::CopiedPages => "copied-pages",
        CounterContractKind::FutureS5SpecificCounters => "future-s5-specific-counters",
    }
}

pub(crate) fn counter_expectation_strength_token(
    strength: CounterExpectationStrength,
) -> &'static str {
    counter_expectation_kind_token(strength)
}
