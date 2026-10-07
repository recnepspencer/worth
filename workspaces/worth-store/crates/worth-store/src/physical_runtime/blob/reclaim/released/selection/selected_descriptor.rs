use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobReclaimDescriptorV3, BlobRecordDenial, BlobRecordKind, OriginalDropReservedV1,
    PersistedRecordIdentity, SelectedRecordContentClass,
};

use crate::physical_runtime::{
    durability::PhysicalReclaimAttempt, CompletedPhysicalMutation,
    PhysicalProtectedRootObservation, PhysicalRecordId, PhysicalRecordReader, RecordByteLimit,
    RecordReadLimits, ServingPhysicalRuntime,
};

use super::super::super::{BlobReclaimFailure, BlobReclaimLimits};

/// A selected route and complete V3 frame read under one protected candidate
/// root. Only this constructor can mint the observation used by owner commit.
pub(in crate::physical_runtime) struct SelectedReleasedDescriptorObservation {
    _reader: PhysicalRecordReader,
    root: PhysicalProtectedRootObservation,
    record: PersistedRecordIdentity,
    descriptor: BlobReclaimDescriptorV3,
    frame_sha256: [u8; 32],
    reservation_record: PersistedRecordIdentity,
    reservation: OriginalDropReservedV1,
    reservation_frame_sha256: [u8; 32],
}

impl SelectedReleasedDescriptorObservation {
    pub(in crate::physical_runtime) fn root(&self) -> PhysicalProtectedRootObservation {
        self.root
    }

    pub(in crate::physical_runtime) const fn record(&self) -> PersistedRecordIdentity {
        self.record
    }

    pub(in crate::physical_runtime) const fn descriptor(&self) -> BlobReclaimDescriptorV3 {
        self.descriptor
    }

    pub(in crate::physical_runtime) const fn frame_sha256(&self) -> [u8; 32] {
        self.frame_sha256
    }

    pub(in crate::physical_runtime) const fn reservation_record(&self) -> PersistedRecordIdentity {
        self.reservation_record
    }

    pub(in crate::physical_runtime) const fn reservation(&self) -> OriginalDropReservedV1 {
        self.reservation
    }

    pub(in crate::physical_runtime) const fn reservation_frame_sha256(&self) -> [u8; 32] {
        self.reservation_frame_sha256
    }
}

pub(in crate::physical_runtime::blob::reclaim) fn observe_selected_descriptor(
    runtime: &ServingPhysicalRuntime,
    attempt: &PhysicalReclaimAttempt,
    completed: &CompletedPhysicalMutation,
    record: PersistedRecordIdentity,
    expected: BlobReclaimDescriptorV3,
    reservation_record: PersistedRecordIdentity,
    expected_reservation: OriginalDropReservedV1,
    limits: BlobReclaimLimits,
) -> Result<SelectedReleasedDescriptorObservation, BlobReclaimFailure> {
    let reader = runtime.capture_released_reclaim_candidate(attempt, completed)?;
    let root = reader.protected_root();
    if root.root().generation().get() != expected.base().candidate_root_generation() {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let descriptor_frame = expected.encode();
    let reservation_frame = expected_reservation.encode();
    let inspected = descriptor_frame
        .len()
        .checked_add(reservation_frame.len())
        .and_then(|length| length.checked_add(2))
        .ok_or(BlobReclaimFailure::InspectedByteBoundExhausted)?;
    if inspected as u64 > limits.maximum_inspected_bytes() {
        return Err(BlobReclaimFailure::InspectedByteBoundExhausted);
    }
    for (selected_record, kind) in [
        (record, BlobRecordKind::ReclaimDescriptorV3),
        (reservation_record, BlobRecordKind::OriginalDropReserved),
    ] {
        if reader
            .selected_content_class(PhysicalRecordId::from_persisted(selected_record))
            .map_err(BlobReclaimFailure::Read)?
            != SelectedRecordContentClass::Blob(kind)
        {
            return Err(BlobReclaimFailure::ConflictingSelectedFate);
        }
    }
    let frame = read_exact_selected_frame(&reader, record, descriptor_frame.len())?;
    let descriptor = BlobReclaimDescriptorV3::decode(&frame).map_err(BlobReclaimFailure::Format)?;
    if descriptor != expected {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let reservation_frame =
        read_exact_selected_frame(&reader, reservation_record, reservation_frame.len())?;
    let reservation =
        OriginalDropReservedV1::decode(&reservation_frame).map_err(BlobReclaimFailure::Format)?;
    if reservation != expected_reservation
        || descriptor.custody().request() != reservation.request()
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    Ok(SelectedReleasedDescriptorObservation {
        _reader: reader,
        root,
        record,
        descriptor,
        frame_sha256: Sha256::digest(&frame).into(),
        reservation_record,
        reservation,
        reservation_frame_sha256: Sha256::digest(&reservation_frame).into(),
    })
}

fn read_exact_selected_frame(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    length: usize,
) -> Result<Vec<u8>, BlobReclaimFailure> {
    let limit = RecordByteLimit::new(
        u32::try_from(length + 1)
            .map_err(|_| BlobReclaimFailure::Format(BlobRecordDenial::FrameTooLarge))?,
    )
    .ok_or(BlobReclaimFailure::Format(BlobRecordDenial::FrameTooLarge))?;
    let mut selected = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(BlobReclaimFailure::Read)?;
    let mut frame = vec![0u8; length];
    let mut used = 0;
    while used < frame.len() {
        let read = selected
            .read_next(&mut frame[used..])
            .map_err(BlobReclaimFailure::Stream)?;
        if read == 0 {
            return Err(BlobReclaimFailure::Format(BlobRecordDenial::Truncated));
        }
        used += read;
    }
    let mut extra = [0u8; 1];
    if selected
        .read_next(&mut extra)
        .map_err(BlobReclaimFailure::Stream)?
        != 0
    {
        return Err(BlobReclaimFailure::Format(BlobRecordDenial::LengthMismatch));
    }
    Ok(frame)
}
