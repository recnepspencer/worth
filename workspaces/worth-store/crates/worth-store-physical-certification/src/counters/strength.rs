use super::{CounterContractKind, CounterExpectationKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CounterStrengthPosture {
    WeakestSufficient,
    ExactnessIsClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CounterStrengthJustification {
    ForbiddenBehaviorMustRemainZero,
    DeterministicEventStructure,
    ReplayIdentity,
    ProfileResourceEnvelope,
    ImplementationSensitiveCost,
}

pub(crate) fn classify_counter_strength(
    kind: CounterContractKind,
    expectation: CounterExpectationKind,
) -> Result<(CounterStrengthPosture, CounterStrengthJustification), OverExactCounterDenied> {
    let justification = counter_strength_justification(kind);
    if justification == CounterStrengthJustification::ImplementationSensitiveCost
        && expectation == CounterExpectationKind::Exact
    {
        return Err(OverExactCounterDenied { kind });
    }
    let posture = match justification {
        CounterStrengthJustification::ForbiddenBehaviorMustRemainZero
        | CounterStrengthJustification::DeterministicEventStructure
        | CounterStrengthJustification::ReplayIdentity => CounterStrengthPosture::ExactnessIsClaim,
        CounterStrengthJustification::ProfileResourceEnvelope
        | CounterStrengthJustification::ImplementationSensitiveCost => {
            CounterStrengthPosture::WeakestSufficient
        }
    };
    Ok((posture, justification))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverExactCounterDenied {
    kind: CounterContractKind,
}

impl OverExactCounterDenied {
    pub const fn kind(self) -> CounterContractKind {
        self.kind
    }
}

const fn counter_strength_justification(kind: CounterContractKind) -> CounterStrengthJustification {
    match kind {
        CounterContractKind::ForbiddenShortcutExact => {
            CounterStrengthJustification::ForbiddenBehaviorMustRemainZero
        }
        CounterContractKind::ActorStepExact => {
            CounterStrengthJustification::DeterministicEventStructure
        }
        CounterContractKind::ReplayIdentityExact => CounterStrengthJustification::ReplayIdentity,
        CounterContractKind::CompactionPublicationPlanCompletions => {
            CounterStrengthJustification::DeterministicEventStructure
        }
        CounterContractKind::BlobChunkCountExact | CounterContractKind::BlobLogicalBytesExact => {
            CounterStrengthJustification::DeterministicEventStructure
        }
        CounterContractKind::ProfileResourceEnvelope => {
            CounterStrengthJustification::ProfileResourceEnvelope
        }
        CounterContractKind::AllocationBytes
        | CounterContractKind::PagePins
        | CounterContractKind::IoQueueDepth
        | CounterContractKind::ResidentBytes
        | CounterContractKind::DirtyPages
        | CounterContractKind::IoInterferenceEvents
        | CounterContractKind::LatchWaits
        | CounterContractKind::EpochRetries
        | CounterContractKind::ProtectedReferences
        | CounterContractKind::Retries
        | CounterContractKind::BlockedReclaimAttempts
        | CounterContractKind::ReplayedPages
        | CounterContractKind::CompactionCandidateRanges
        | CounterContractKind::CopiedPages
        | CounterContractKind::FutureS5SpecificCounters => {
            CounterStrengthJustification::ImplementationSensitiveCost
        }
    }
}
