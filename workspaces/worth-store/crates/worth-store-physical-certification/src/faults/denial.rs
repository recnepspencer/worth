use super::event::PhysicalFaultEventKind;
use super::locus::{ExpectedFaultLocalization, PhysicalArtifactFaultLocus};
use crate::{PhysicalBoundarySeam, PhysicalBoundaryYieldpoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultObservedBoundaryKind {
    FreshRuntimeCrashRecovery,
    NoFaultProductionBoundary,
    IoPressureBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaultDeliveryDenial {
    PrivateMutationDenied,
    ArbitraryByteScribbleDenied,
    SameProcessCrashDenied,
    PostDecodeCorruptionDenied,
    AmbiguousFaultLocusDenied,
    MissingFreshRuntimeEvidence,
    MissingObservedFaultBoundary,
    NoFaultParityMismatch,
    FaultHasNoProductionStorageInjection(PhysicalFaultEventKind),
    ProductionStorageFaultWasNotExecuted,
    UnboundFaultYieldpoint {
        scheduled_yieldpoint: String,
        delivery_yieldpoint: String,
    },
    FaultYieldpointSeamMismatch {
        expected: PhysicalBoundarySeam,
        actual: PhysicalBoundarySeam,
    },
    ObservedFaultBoundaryMismatch {
        expected: ExpectedFaultLocalization,
        actual: FaultObservedBoundaryKind,
    },
}

pub(crate) fn deny_unbound_fault_yieldpoint(
    scheduled_yieldpoint: &str,
    delivery_yieldpoint: &PhysicalBoundaryYieldpoint,
) -> FaultDeliveryDenial {
    FaultDeliveryDenial::UnboundFaultYieldpoint {
        scheduled_yieldpoint: scheduled_yieldpoint.to_owned(),
        delivery_yieldpoint: delivery_yieldpoint.name().to_owned(),
    }
}

pub(crate) fn require_unambiguous_locus(
    locus: &PhysicalArtifactFaultLocus,
) -> Result<(), FaultDeliveryDenial> {
    if locus.is_ambiguous() {
        return Err(FaultDeliveryDenial::AmbiguousFaultLocusDenied);
    }
    Ok(())
}
