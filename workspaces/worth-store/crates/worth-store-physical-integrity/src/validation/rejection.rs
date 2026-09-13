use crate::localization::{PhysicalByteRange, PhysicalDamageLocalization};

use super::PhysicalArtifactScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityVersionAxis {
    EnvelopeSchema,
    PhysicalFormat,
    PhysicalWorkObligation,
    WalFrame,
    CheckpointRecordSchema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedPhysicalIntegrityVersion {
    scope: PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
}

impl UnsupportedPhysicalIntegrityVersion {
    pub const fn new(
        scope: PhysicalArtifactScope,
        axis: PhysicalIntegrityVersionAxis,
        observed: u32,
    ) -> Self {
        Self {
            scope,
            axis,
            observed,
        }
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn axis(self) -> PhysicalIntegrityVersionAxis {
        self.axis
    }

    pub const fn observed(self) -> u32 {
        self.observed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownPhysicalIntegrityCause {
    ExpectedArtifactAbsent,
    UnrecognizedArtifact,
    ExpectedScopeUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPhysicalIntegrityPosture {
    scope: PhysicalArtifactScope,
    cause: UnknownPhysicalIntegrityCause,
}

impl UnknownPhysicalIntegrityPosture {
    pub const fn new(scope: PhysicalArtifactScope, cause: UnknownPhysicalIntegrityCause) -> Self {
        Self { scope, cause }
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn cause(self) -> UnknownPhysicalIntegrityCause {
        self.cause
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndeterminatePhysicalIntegrityCause {
    SourceChangedDuringInspection,
    ObservationBoundExhausted,
    StableRangeNotProven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndeterminatePhysicalIntegrityPosture {
    scope: PhysicalArtifactScope,
    cause: IndeterminatePhysicalIntegrityCause,
    observed_range: Option<PhysicalByteRange>,
}

impl IndeterminatePhysicalIntegrityPosture {
    pub const fn new(
        scope: PhysicalArtifactScope,
        cause: IndeterminatePhysicalIntegrityCause,
        observed_range: Option<PhysicalByteRange>,
    ) -> Self {
        Self {
            scope,
            cause,
            observed_range,
        }
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn cause(self) -> IndeterminatePhysicalIntegrityCause {
        self.cause
    }

    pub const fn observed_range(self) -> Option<PhysicalByteRange> {
        self.observed_range
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityRejection {
    Damaged(PhysicalDamageLocalization),
    Unsupported(UnsupportedPhysicalIntegrityVersion),
    Unknown(UnknownPhysicalIntegrityPosture),
    Indeterminate(IndeterminatePhysicalIntegrityPosture),
}

impl PhysicalIntegrityRejection {
    pub const fn scope(self) -> PhysicalArtifactScope {
        match self {
            Self::Damaged(localization) => localization.scope(),
            Self::Unsupported(posture) => posture.scope(),
            Self::Unknown(posture) => posture.scope(),
            Self::Indeterminate(posture) => posture.scope(),
        }
    }
}
