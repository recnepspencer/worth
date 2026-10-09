use super::btree::{declare_btree_invariant_suite, BTreeInvariantSuite};
use super::counter_evidence::StrategyCounterEvidence;
use super::counter_path::derive_strategy_counter_evidence;
use super::declaration::StrategyDeclaration;
use super::lsm::{declare_lsm_invariant_suite, LsmInvariantSuite};
use super::{LayoutStrategyFamily, StrategyDenial};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyLookupInvariant {
    SeparatorDirectedLookup,
    NewestRunLookup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyPublicationInvariant {
    RootPublication,
    ManifestPublication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyMutationInvariant {
    SplitMaintainsOccupancy,
    TombstonesSurviveCompaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyIntegrityInvariant {
    ChecksumLocalizesCorruption,
    ManifestBindsRunDigests,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyRecoveryInvariant {
    StableReadReplay,
    WalReplayRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrategyCounterProfile {
    point_lookups: u16,
    range_lookups: u16,
    wal_replays: u16,
    publications: u16,
    maintenance_reads: u16,
}

impl StrategyCounterProfile {
    pub(crate) const fn new(
        point_lookups: u16,
        range_lookups: u16,
        wal_replays: u16,
        publications: u16,
        maintenance_reads: u16,
    ) -> Self {
        Self {
            point_lookups,
            range_lookups,
            wal_replays,
            publications,
            maintenance_reads,
        }
    }

    pub const fn point_lookups(self) -> u16 {
        self.point_lookups
    }

    pub const fn range_lookups(self) -> u16 {
        self.range_lookups
    }

    pub const fn wal_replays(self) -> u16 {
        self.wal_replays
    }

    pub const fn publications(self) -> u16 {
        self.publications
    }

    pub const fn maintenance_reads(self) -> u16 {
        self.maintenance_reads
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// The admitted suite is a cold, immutable law bundle retained behind the registry snapshot's
// shared handle. Keeping both strategy-native suites inline preserves Copy semantics for the
// production strategy capability and avoids adding allocation to every strategy use.
#[allow(clippy::large_enum_variant)]
enum StrategySpecificInvariantSuite {
    BTree(BTreeInvariantSuite),
    Lsm(LsmInvariantSuite),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrategyInvariantSuite {
    declaration: StrategyDeclaration,
    lookup: StrategyLookupInvariant,
    publication: StrategyPublicationInvariant,
    mutation: StrategyMutationInvariant,
    integrity: StrategyIntegrityInvariant,
    recovery: StrategyRecoveryInvariant,
    counter_evidence: StrategyCounterEvidence,
    specific: StrategySpecificInvariantSuite,
}

#[derive(Debug, PartialEq, Eq)]
enum StrategyInvariantAdmissionCase {
    Success(Box<StrategyInvariantSuite>),
    Denied(StrategyDenial),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StrategyInvariantAdmissionOutcome {
    case: StrategyInvariantAdmissionCase,
}

impl StrategyInvariantAdmissionOutcome {
    fn admitted(value: StrategyInvariantSuite) -> Self {
        Self::from_owner_payload(StrategyInvariantAdmissionCase::Success(Box::new(value)))
    }

    fn denied(value: StrategyDenial) -> Self {
        Self::from_owner_payload(StrategyInvariantAdmissionCase::Denied(value))
    }

    fn from_owner_payload(case: StrategyInvariantAdmissionCase) -> Self {
        Self { case }
    }

    fn into_owner_payload(self) -> StrategyInvariantAdmissionCase {
        self.case
    }
}

impl StrategyInvariantAdmissionOutcome {
    pub(crate) fn into_admitted(self) -> Result<AdmittedStrategyInvariants, StrategyDenial> {
        match self.into_owner_payload() {
            StrategyInvariantAdmissionCase::Success(suite) => {
                Ok(AdmittedStrategyInvariants { suite: *suite })
            }
            StrategyInvariantAdmissionCase::Denied(denial) => Err(denial),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AdmittedStrategyInvariants {
    suite: StrategyInvariantSuite,
}

impl AdmittedStrategyInvariants {
    pub(crate) const fn suite(self) -> StrategyInvariantSuite {
        self.suite
    }
}

impl StrategyInvariantSuite {
    pub(crate) fn declare(declaration: StrategyDeclaration) -> StrategyInvariantAdmissionOutcome {
        match Self::declare_result(declaration) {
            Ok(suite) => StrategyInvariantAdmissionOutcome::admitted(suite),
            Err(denial) => StrategyInvariantAdmissionOutcome::denied(denial),
        }
    }

    fn declare_result(declaration: StrategyDeclaration) -> Result<Self, StrategyDenial> {
        match declaration.family() {
            LayoutStrategyFamily::BTreeRange => Ok(Self {
                declaration,
                lookup: StrategyLookupInvariant::SeparatorDirectedLookup,
                publication: StrategyPublicationInvariant::RootPublication,
                mutation: StrategyMutationInvariant::SplitMaintainsOccupancy,
                integrity: StrategyIntegrityInvariant::ChecksumLocalizesCorruption,
                recovery: StrategyRecoveryInvariant::StableReadReplay,
                counter_evidence: derive_strategy_counter_evidence(declaration),
                specific: StrategySpecificInvariantSuite::BTree(declare_btree_invariant_suite(
                    declaration,
                )?),
            }),
            LayoutStrategyFamily::BaselineLsmWriteOptimized => Ok(Self {
                declaration,
                lookup: StrategyLookupInvariant::NewestRunLookup,
                publication: StrategyPublicationInvariant::ManifestPublication,
                mutation: StrategyMutationInvariant::TombstonesSurviveCompaction,
                integrity: StrategyIntegrityInvariant::ManifestBindsRunDigests,
                recovery: StrategyRecoveryInvariant::WalReplayRecovery,
                counter_evidence: derive_strategy_counter_evidence(declaration),
                specific: StrategySpecificInvariantSuite::Lsm(declare_lsm_invariant_suite(
                    declaration,
                )?),
            }),
            _ => Err(StrategyDenial::InvariantSuiteNotAvailableForFamily),
        }
    }

    pub const fn family(self) -> LayoutStrategyFamily {
        self.declaration.family()
    }

    pub const fn lookup_invariant(self) -> StrategyLookupInvariant {
        self.lookup
    }

    pub const fn publication_invariant(self) -> StrategyPublicationInvariant {
        self.publication
    }

    pub const fn mutation_invariant(self) -> StrategyMutationInvariant {
        self.mutation
    }

    pub const fn integrity_invariant(self) -> StrategyIntegrityInvariant {
        self.integrity
    }

    pub const fn recovery_invariant(self) -> StrategyRecoveryInvariant {
        self.recovery
    }

    pub const fn counter_evidence(self) -> StrategyCounterEvidence {
        self.counter_evidence
    }

    pub const fn require_btree_suite(self) -> Result<BTreeInvariantSuite, StrategyDenial> {
        match self.specific {
            StrategySpecificInvariantSuite::BTree(suite) => Ok(suite),
            StrategySpecificInvariantSuite::Lsm(_) => {
                Err(StrategyDenial::InvariantSuiteNotAvailableForFamily)
            }
        }
    }

    pub const fn require_lsm_suite(self) -> Result<LsmInvariantSuite, StrategyDenial> {
        match self.specific {
            StrategySpecificInvariantSuite::Lsm(suite) => Ok(suite),
            StrategySpecificInvariantSuite::BTree(_) => {
                Err(StrategyDenial::InvariantSuiteNotAvailableForFamily)
            }
        }
    }
}
