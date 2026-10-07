//! A V3 control may be retired from the final selected root. Its earlier
//! candidate root remains usable only through an exact checked root edge.

use sha2::Digest;
use worth_store_physical_format::{
    BlobRecordKind, CurrentPhysicalRecordPlacement, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration, SelectedRecordContentClass, BLOB_CONTROL_FRAME_MAX_BYTES,
};

use super::{
    ordered_root_history::VerifiedReleasedRootEdge,
    physics_budget::{ExceededPhysicsBound, PhysicsAllowance},
    released_v3_inventory_transition::transcript,
    ReleasedInventoryView, WitnessedSelectedControlFrame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressedReleasedControlDenial {
    ResultRoot,
    Route,
    Frame,
    /// The frame would retain more bytes than its caller admitted.
    Bound(ExceededPhysicsBound),
    /// The frame's retained size passes every count, so no bound states it.
    CountOverflow,
    /// The frame's copy could not be allocated.
    Allocation,
}

/// This token joins integrity-validated control bytes to a complete addressed
/// result inventory and a checked C.9 V3 edge. It is not a Store media seal.
#[derive(Debug, Clone)]
pub struct VerifiedAddressedReleasedControlFrame {
    candidate_root_frame_sha256: [u8; 32],
    record: PersistedRecordIdentity,
    kind: BlobRecordKind,
    payload_sha256: [u8; 32],
    bytes: Box<[u8]>,
    retained_bytes: u64,
}

impl VerifiedAddressedReleasedControlFrame {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.bytes.len()).ok()
    }

    pub fn admit(
        edge: &VerifiedReleasedRootEdge,
        result: ReleasedInventoryView<'_>,
        frame: &WitnessedSelectedControlFrame,
        kind: BlobRecordKind,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        remaining_retained_bytes: u64,
    ) -> Result<Self, AddressedReleasedControlDenial> {
        let retained_bytes = (frame.bytes().len() as u64)
            .checked_add(std::mem::size_of::<Self>() as u64)
            .ok_or(AddressedReleasedControlDenial::CountOverflow)?;
        PhysicsAllowance::retained_bytes(remaining_retained_bytes)
            .admit(retained_bytes)
            .map_err(AddressedReleasedControlDenial::Bound)?;
        if transcript(result, format, maximum_entries)
            .map_err(|_| AddressedReleasedControlDenial::ResultRoot)?
            != edge.transition().result_topology()
            || frame.bytes().len() > BLOB_CONTROL_FRAME_MAX_BYTES
        {
            return Err(AddressedReleasedControlDenial::ResultRoot);
        }
        let placement = frame.selected_placement();
        if !matches!(placement,
            CurrentPhysicalRecordPlacement::Extent(extent)
                if extent.content_class() == SelectedRecordContentClass::Blob(kind)
        ) || result
            .routes
            .binary_search_by_key(&placement.record(), |route| route.record())
            .ok()
            .is_none_or(|index| result.routes[index] != placement)
        {
            return Err(AddressedReleasedControlDenial::Route);
        }
        let payload_sha256 = sha2::Sha256::digest(frame.bytes()).into();
        if payload_sha256 != frame.selected_payload_sha256() {
            return Err(AddressedReleasedControlDenial::Frame);
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(frame.bytes().len())
            .map_err(|_| AddressedReleasedControlDenial::Allocation)?;
        bytes.extend_from_slice(frame.bytes());
        Ok(Self {
            candidate_root_frame_sha256: edge.result_root_frame_sha256(),
            record: placement.record(),
            kind,
            payload_sha256,
            bytes: bytes.into_boxed_slice(),
            retained_bytes,
        })
    }

    pub const fn candidate_root_frame_sha256(&self) -> [u8; 32] {
        self.candidate_root_frame_sha256
    }
    pub const fn record(&self) -> PersistedRecordIdentity {
        self.record
    }
    pub const fn kind(&self) -> BlobRecordKind {
        self.kind
    }
    pub const fn payload_sha256(&self) -> [u8; 32] {
        self.payload_sha256
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }
}
