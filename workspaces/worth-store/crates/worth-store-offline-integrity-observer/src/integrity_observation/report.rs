use worth_foundational::{
    PhysicalArtifactFamily, PhysicalArtifactGeneration, PhysicalArtifactIdentity, PhysicalByteRange,
};

use super::{
    OfflineIntegrityObservationCounters, OfflineIntegrityObservationLimits,
    OfflineIntegrityOutcome, OfflineIntegrityProtocolContext, OFFLINE_OBSERVER_ROLE_IDENTITY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityReportCompleteness {
    Complete,
    BoundExhausted,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineArtifactFamily {
    Declared(PhysicalArtifactFamily),
    Unrecognized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfflineArtifactDuplicateEvidence {
    PhysicalAlias { first_path: Box<str> },
    SemanticIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineArtifactObservation {
    relative_path: Box<str>,
    family: OfflineArtifactFamily,
    identity: PhysicalArtifactIdentity,
    generation: PhysicalArtifactGeneration,
    range: Option<PhysicalByteRange>,
    outcome: OfflineIntegrityOutcome,
    duplicates: Vec<OfflineArtifactDuplicateEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineIntegrityReport {
    protocol_context: OfflineIntegrityProtocolContext,
    store_identity: Option<Box<str>>,
    declared_limits: OfflineIntegrityObservationLimits,
    counters: OfflineIntegrityObservationCounters,
    completeness: OfflineIntegrityReportCompleteness,
    artifacts: Vec<OfflineArtifactObservation>,
}

impl OfflineArtifactObservation {
    pub(crate) fn new(
        relative_path: impl Into<Box<str>>,
        family: OfflineArtifactFamily,
        identity: PhysicalArtifactIdentity,
        generation: PhysicalArtifactGeneration,
        range: Option<PhysicalByteRange>,
        outcome: OfflineIntegrityOutcome,
    ) -> Self {
        Self {
            relative_path: relative_path.into(),
            family,
            identity,
            generation,
            range,
            outcome,
            duplicates: Vec::new(),
        }
    }

    pub(crate) fn with_duplicate(mut self, duplicate: OfflineArtifactDuplicateEvidence) -> Self {
        self.duplicates.push(duplicate);
        self
    }

    pub(crate) fn with_outcome(mut self, outcome: OfflineIntegrityOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }
    pub const fn family(&self) -> OfflineArtifactFamily {
        self.family
    }
    pub const fn identity(&self) -> &PhysicalArtifactIdentity {
        &self.identity
    }
    pub const fn generation(&self) -> PhysicalArtifactGeneration {
        self.generation
    }
    pub const fn range(&self) -> Option<PhysicalByteRange> {
        self.range
    }
    pub const fn outcome(&self) -> &OfflineIntegrityOutcome {
        &self.outcome
    }
    pub fn duplicates(&self) -> &[OfflineArtifactDuplicateEvidence] {
        &self.duplicates
    }
}

impl From<PhysicalArtifactFamily> for OfflineArtifactFamily {
    fn from(value: PhysicalArtifactFamily) -> Self {
        Self::Declared(value)
    }
}

impl PartialEq<PhysicalArtifactFamily> for OfflineArtifactFamily {
    fn eq(&self, other: &PhysicalArtifactFamily) -> bool {
        matches!(self, Self::Declared(family) if family == other)
    }
}

impl OfflineArtifactFamily {
    pub const fn declared(self) -> Option<PhysicalArtifactFamily> {
        match self {
            Self::Declared(family) => Some(family),
            Self::Unrecognized => None,
        }
    }
}

impl OfflineIntegrityReport {
    pub(crate) fn new(
        protocol_context: OfflineIntegrityProtocolContext,
        store_identity: Option<Box<str>>,
        declared_limits: OfflineIntegrityObservationLimits,
        counters: OfflineIntegrityObservationCounters,
        completeness: OfflineIntegrityReportCompleteness,
        artifacts: Vec<OfflineArtifactObservation>,
    ) -> Self {
        Self {
            protocol_context,
            store_identity,
            declared_limits,
            counters,
            completeness,
            artifacts,
        }
    }

    pub const fn protocol_context(&self) -> &OfflineIntegrityProtocolContext {
        &self.protocol_context
    }
    pub const fn role_identity(&self) -> &'static str {
        OFFLINE_OBSERVER_ROLE_IDENTITY
    }
    pub fn store_identity(&self) -> Option<&str> {
        self.store_identity.as_deref()
    }
    pub const fn declared_limits(&self) -> OfflineIntegrityObservationLimits {
        self.declared_limits
    }
    pub const fn counters(&self) -> &OfflineIntegrityObservationCounters {
        &self.counters
    }
    pub const fn completeness(&self) -> OfflineIntegrityReportCompleteness {
        self.completeness
    }
    pub fn artifacts(&self) -> &[OfflineArtifactObservation] {
        &self.artifacts
    }

    pub(crate) fn counters_mut(&mut self) -> &mut OfflineIntegrityObservationCounters {
        &mut self.counters
    }
}
