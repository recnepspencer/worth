use worth_foundational::{BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator};
use worth_store_physical_certification::{
    ExpectedFaultLocalization, FaultDeliveryAttempt, ObservedFaultBoundary,
    PhysicalArtifactFaultLocus, PhysicalFaultFieldKind, PhysicalFaultOffset,
};

pub fn wal_frame_payload_fault_locus() -> PhysicalArtifactFaultLocus {
    PhysicalArtifactFaultLocus::wal_frame(
        BoundaryArtifactLocator::new(BoundaryArtifactId::new(45), BoundaryArtifactField::Payload),
        PhysicalFaultFieldKind::ChecksumProtectedPayload,
        PhysicalFaultOffset::at(24),
        ExpectedFaultLocalization::PreDecodeBoundary,
    )
}

pub fn page_generation_fault_locus() -> PhysicalArtifactFaultLocus {
    PhysicalArtifactFaultLocus::page_image(
        BoundaryArtifactLocator::new(BoundaryArtifactId::new(52), BoundaryArtifactField::Payload),
        PhysicalFaultFieldKind::GenerationField,
        PhysicalFaultOffset::at(8),
        ExpectedFaultLocalization::PhysicalIntegrityBoundary,
    )
}

pub fn crash_recovery_fault_locus() -> PhysicalArtifactFaultLocus {
    PhysicalArtifactFaultLocus::crash_recovery_runtime(
        BoundaryArtifactLocator::new(BoundaryArtifactId::new(60), BoundaryArtifactField::Basis),
        ExpectedFaultLocalization::FreshRuntimeRecoveryBoundary,
    )
}

pub fn io_pressure_fault_locus() -> PhysicalArtifactFaultLocus {
    PhysicalArtifactFaultLocus::page_image(
        BoundaryArtifactLocator::new(BoundaryArtifactId::new(70), BoundaryArtifactField::Payload),
        PhysicalFaultFieldKind::SlotState,
        PhysicalFaultOffset::at(16),
        ExpectedFaultLocalization::ProductionDriverBoundary,
    )
}

pub const fn observed_io_pressure_boundary() -> ObservedFaultBoundary {
    ObservedFaultBoundary::io_pressure_boundary()
}

pub const fn private_mutation_fault_attempt_fixture() -> FaultDeliveryAttempt {
    FaultDeliveryAttempt::private_mutation()
}

pub const fn arbitrary_byte_scribble_fault_attempt_fixture() -> FaultDeliveryAttempt {
    FaultDeliveryAttempt::arbitrary_byte_scribble()
}

pub const fn same_process_crash_fault_attempt_fixture() -> FaultDeliveryAttempt {
    FaultDeliveryAttempt::same_process_crash()
}

pub const fn post_decode_corruption_fault_attempt_fixture() -> FaultDeliveryAttempt {
    FaultDeliveryAttempt::post_decode_corruption()
}

pub const fn ambiguous_locus_fault_attempt_fixture() -> FaultDeliveryAttempt {
    FaultDeliveryAttempt::ambiguous_locus()
}
