use std::num::{NonZeroU32, NonZeroU64};

use super::super::{
    PhysicalOperationAllocationScope, PhysicalResidencyDimension, PhysicalResidencyLimits,
    PhysicalResidencyLimitsAdmissionDenial, PhysicalSpeculativeWorkKind,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalResidencyLimitsBuilder {
    total_bytes: Option<NonZeroU64>,
    resident_bytes: Option<NonZeroU64>,
    metadata_bytes: Option<NonZeroU64>,
    frame_entries: Option<NonZeroU32>,
    pinned_frames: Option<NonZeroU32>,
    pin_leases: Option<NonZeroU32>,
    dirty_frames: Option<NonZeroU32>,
    dirty_replacement_bytes: Option<NonZeroU64>,
    operation_bytes: Option<NonZeroU64>,
    scope_bytes: [Option<NonZeroU64>; 7],
    speculative_frames: [Option<NonZeroU32>; 3],
    progress_headroom_bytes: u64,
}

impl PhysicalResidencyLimitsBuilder {
    pub const fn total_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.total_bytes = Some(bytes);
        self
    }

    pub const fn resident_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.resident_bytes = Some(bytes);
        self
    }

    pub const fn metadata_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.metadata_bytes = Some(bytes);
        self
    }

    pub const fn frame_entries(mut self, entries: NonZeroU32) -> Self {
        self.frame_entries = Some(entries);
        self
    }

    pub const fn pinned_frames(mut self, frames: NonZeroU32) -> Self {
        self.pinned_frames = Some(frames);
        self
    }

    pub const fn pin_leases(mut self, leases: NonZeroU32) -> Self {
        self.pin_leases = Some(leases);
        self
    }

    pub const fn dirty_frames(mut self, frames: NonZeroU32) -> Self {
        self.dirty_frames = Some(frames);
        self
    }

    pub const fn dirty_replacement_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.dirty_replacement_bytes = Some(bytes);
        self
    }

    pub const fn operation_bytes(mut self, bytes: NonZeroU64) -> Self {
        self.operation_bytes = Some(bytes);
        self
    }

    pub const fn scope_bytes(
        mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: NonZeroU64,
    ) -> Self {
        self.scope_bytes[scope.index()] = Some(bytes);
        self
    }

    pub const fn speculative_frames(
        mut self,
        kind: PhysicalSpeculativeWorkKind,
        frames: NonZeroU32,
    ) -> Self {
        self.speculative_frames[kind.index()] = Some(frames);
        self
    }

    pub const fn progress_headroom_bytes(mut self, bytes: u64) -> Self {
        self.progress_headroom_bytes = bytes;
        self
    }

    pub fn admit(
        self,
        page_bytes: NonZeroU64,
    ) -> Result<PhysicalResidencyLimits, PhysicalResidencyLimitsAdmissionDenial> {
        let total_bytes = required(self.total_bytes, PhysicalResidencyDimension::TotalBytes)?;
        let resident_bytes = required(
            self.resident_bytes,
            PhysicalResidencyDimension::ResidentBytes,
        )?;
        let metadata_bytes = required(
            self.metadata_bytes,
            PhysicalResidencyDimension::MetadataBytes,
        )?;
        let frame_entries = required(self.frame_entries, PhysicalResidencyDimension::FrameEntries)?;
        let pinned_frames = required(self.pinned_frames, PhysicalResidencyDimension::PinnedFrames)?;
        let pin_leases = required(self.pin_leases, PhysicalResidencyDimension::PinLeases)?;
        let dirty_frames = required(self.dirty_frames, PhysicalResidencyDimension::DirtyFrames)?;
        let dirty_replacement_bytes = required(
            self.dirty_replacement_bytes,
            PhysicalResidencyDimension::DirtyReplacementBytes,
        )?;
        let operation_bytes = required(
            self.operation_bytes,
            PhysicalResidencyDimension::OperationBytes,
        )?;
        let scope_bytes = admit_scopes(self.scope_bytes, operation_bytes)?;
        let speculative_frames = admit_speculation(self.speculative_frames)?;
        let admitted = PhysicalResidencyLimits {
            total_bytes,
            resident_bytes,
            metadata_bytes,
            frame_entries,
            pinned_frames,
            pin_leases,
            dirty_frames,
            dirty_replacement_bytes,
            operation_bytes,
            scope_bytes,
            speculative_frames,
            progress_headroom_bytes: self.progress_headroom_bytes,
        };
        validate_relationships(admitted, page_bytes)?;
        Ok(admitted)
    }
}

fn validate_relationships(
    limits: PhysicalResidencyLimits,
    page_bytes: NonZeroU64,
) -> Result<(), PhysicalResidencyLimitsAdmissionDenial> {
    for (dimension, declared) in [
        (
            PhysicalResidencyDimension::ResidentBytes,
            limits.resident_bytes(),
        ),
        (
            PhysicalResidencyDimension::MetadataBytes,
            limits.metadata_bytes(),
        ),
        (
            PhysicalResidencyDimension::DirtyReplacementBytes,
            limits.dirty_replacement_bytes(),
        ),
        (
            PhysicalResidencyDimension::OperationBytes,
            limits.operation_bytes(),
        ),
    ] {
        if declared > limits.total_bytes() {
            return Err(
                PhysicalResidencyLimitsAdmissionDenial::CategoryExceedsTotal {
                    dimension,
                    declared,
                    total: limits.total_bytes(),
                },
            );
        }
    }
    validate_counts(limits)?;
    validate_page(limits, page_bytes)
}

fn validate_counts(
    limits: PhysicalResidencyLimits,
) -> Result<(), PhysicalResidencyLimitsAdmissionDenial> {
    validate_count(
        PhysicalResidencyDimension::PinnedFrames,
        limits.pinned_frames(),
        limits.frame_entries(),
    )?;
    validate_count(
        PhysicalResidencyDimension::DirtyFrames,
        limits.dirty_frames(),
        limits.frame_entries(),
    )?;
    for kind in speculative_kinds() {
        validate_count(
            PhysicalResidencyDimension::SpeculativeFrames(kind),
            limits.speculative_frames(kind),
            limits.frame_entries(),
        )?;
    }
    Ok(())
}

fn validate_page(
    limits: PhysicalResidencyLimits,
    page_bytes: NonZeroU64,
) -> Result<(), PhysicalResidencyLimitsAdmissionDenial> {
    if page_bytes.get() > limits.resident_bytes() {
        return Err(
            PhysicalResidencyLimitsAdmissionDenial::PageExceedsResidentBytes {
                page: page_bytes.get(),
                resident: limits.resident_bytes(),
            },
        );
    }
    if page_bytes.get() > limits.operation_bytes() {
        return Err(
            PhysicalResidencyLimitsAdmissionDenial::PageExceedsOperationBytes {
                page: page_bytes.get(),
                operation: limits.operation_bytes(),
            },
        );
    }
    if page_bytes.get() > limits.dirty_replacement_bytes() {
        return Err(
            PhysicalResidencyLimitsAdmissionDenial::PageExceedsDirtyReplacementBytes {
                page: page_bytes.get(),
                dirty_replacement: limits.dirty_replacement_bytes(),
            },
        );
    }
    Ok(())
}

fn required<T: Copy>(
    value: Option<T>,
    dimension: PhysicalResidencyDimension,
) -> Result<T, PhysicalResidencyLimitsAdmissionDenial> {
    value.ok_or(PhysicalResidencyLimitsAdmissionDenial::Missing(dimension))
}

fn admit_scopes(
    declarations: [Option<NonZeroU64>; 7],
    operation_bytes: NonZeroU64,
) -> Result<[NonZeroU64; 7], PhysicalResidencyLimitsAdmissionDenial> {
    let mut admitted = [operation_bytes; 7];
    for scope in operation_scopes() {
        let bytes = required(
            declarations[scope.index()],
            PhysicalResidencyDimension::OperationScope(scope),
        )?;
        if bytes > operation_bytes {
            return Err(
                PhysicalResidencyLimitsAdmissionDenial::ScopeExceedsOperation {
                    scope,
                    declared: bytes.get(),
                    operation: operation_bytes.get(),
                },
            );
        }
        admitted[scope.index()] = bytes;
    }
    Ok(admitted)
}

fn admit_speculation(
    declarations: [Option<NonZeroU32>; 3],
) -> Result<[NonZeroU32; 3], PhysicalResidencyLimitsAdmissionDenial> {
    let mut admitted = [NonZeroU32::MIN; 3];
    for kind in speculative_kinds() {
        admitted[kind.index()] = required(
            declarations[kind.index()],
            PhysicalResidencyDimension::SpeculativeFrames(kind),
        )?;
    }
    Ok(admitted)
}

fn validate_count(
    dimension: PhysicalResidencyDimension,
    declared: u32,
    frame_entries: u32,
) -> Result<(), PhysicalResidencyLimitsAdmissionDenial> {
    if declared > frame_entries {
        Err(
            PhysicalResidencyLimitsAdmissionDenial::CountExceedsFrameEntries {
                dimension,
                declared,
                frame_entries,
            },
        )
    } else {
        Ok(())
    }
}

const fn operation_scopes() -> [PhysicalOperationAllocationScope; 7] {
    use PhysicalOperationAllocationScope as Scope;
    [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ]
}

const fn speculative_kinds() -> [PhysicalSpeculativeWorkKind; 3] {
    use PhysicalSpeculativeWorkKind as Kind;
    [Kind::ReadAhead, Kind::Prefetch, Kind::WriteBehind]
}
