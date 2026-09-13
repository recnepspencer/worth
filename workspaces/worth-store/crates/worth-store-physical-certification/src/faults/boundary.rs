use super::denial::{FaultDeliveryDenial, FaultObservedBoundaryKind};
use crate::{FreshRuntimeCrashRecoveryEvidence, ProductionBoundaryDriverTrace};
use worth_foundational::FoundationalBoundaryEvidenceLocality;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoFaultProductionBoundaryParity {
    ordinary_trace: ProductionBoundaryDriverTrace,
    control_trace: ProductionBoundaryDriverTrace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservedFaultBoundary {
    FreshRuntimeCrashRecovery {
        locality: FoundationalBoundaryEvidenceLocality,
    },
    NoFaultProductionBoundary {
        parity: NoFaultProductionBoundaryParity,
        locality: FoundationalBoundaryEvidenceLocality,
    },
    IoPressureBoundary {
        locality: FoundationalBoundaryEvidenceLocality,
    },
}

impl NoFaultProductionBoundaryParity {
    pub fn from_traces(
        ordinary_trace: ProductionBoundaryDriverTrace,
        control_trace: ProductionBoundaryDriverTrace,
    ) -> Result<Self, FaultDeliveryDenial> {
        if ordinary_trace != control_trace {
            return Err(FaultDeliveryDenial::NoFaultParityMismatch);
        }
        Ok(Self {
            ordinary_trace,
            control_trace,
        })
    }

    pub const fn ordinary_trace(&self) -> &ProductionBoundaryDriverTrace {
        &self.ordinary_trace
    }

    pub const fn control_trace(&self) -> &ProductionBoundaryDriverTrace {
        &self.control_trace
    }
}

impl ObservedFaultBoundary {
    pub const fn fresh_runtime_crash_recovery(
        _evidence: &FreshRuntimeCrashRecoveryEvidence,
    ) -> Self {
        Self::FreshRuntimeCrashRecovery {
            locality: FoundationalBoundaryEvidenceLocality::RestoredReadmitted,
        }
    }

    pub const fn no_fault_production_boundary(parity: NoFaultProductionBoundaryParity) -> Self {
        Self::NoFaultProductionBoundary {
            parity,
            locality: FoundationalBoundaryEvidenceLocality::Current,
        }
    }

    pub const fn io_pressure_boundary() -> Self {
        Self::IoPressureBoundary {
            locality: FoundationalBoundaryEvidenceLocality::Current,
        }
    }

    pub const fn locality(&self) -> FoundationalBoundaryEvidenceLocality {
        match self {
            Self::FreshRuntimeCrashRecovery { locality }
            | Self::NoFaultProductionBoundary { locality, .. }
            | Self::IoPressureBoundary { locality } => *locality,
        }
    }

    pub const fn boundary_kind(&self) -> FaultObservedBoundaryKind {
        match self {
            Self::FreshRuntimeCrashRecovery { .. } => {
                FaultObservedBoundaryKind::FreshRuntimeCrashRecovery
            }
            Self::NoFaultProductionBoundary { .. } => {
                FaultObservedBoundaryKind::NoFaultProductionBoundary
            }
            Self::IoPressureBoundary { .. } => FaultObservedBoundaryKind::IoPressureBoundary,
        }
    }

    pub const fn no_fault_parity(&self) -> Option<&NoFaultProductionBoundaryParity> {
        match self {
            Self::NoFaultProductionBoundary { parity, .. } => Some(parity),
            _ => None,
        }
    }
}
