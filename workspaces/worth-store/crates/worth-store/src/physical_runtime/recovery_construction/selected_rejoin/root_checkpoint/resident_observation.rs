//! Resident-admitted reads for the independent V2 root/checkpoint observation.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::{
    certificate_stream, observe_parts, Denial, ObservedCheckpointSourceRoot, ObservedRootCheckpoint,
};
use crate::physical_runtime::recovery_construction::selected_rejoin::resident::{
    discovery_allocation_denial, StoreRejoinResidentLedger,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_recovery_physics::VerifiedSelectedReleaseHeadCustodyV2;

pub(super) struct RootCheckpointReader<'a, 'b> {
    discovery: &'a mut BoundedRecoveryFilesystemDiscovery,
    resident: Option<&'b mut StoreRejoinResidentLedger>,
}

impl<'a, 'b> RootCheckpointReader<'a, 'b> {
    pub(super) fn legacy(discovery: &'a mut BoundedRecoveryFilesystemDiscovery) -> Self {
        Self {
            discovery,
            resident: None,
        }
    }

    pub(super) fn resident(
        discovery: &'a mut BoundedRecoveryFilesystemDiscovery,
        resident: &'b mut StoreRejoinResidentLedger,
    ) -> Self {
        Self {
            discovery,
            resident: Some(resident),
        }
    }

    pub(super) fn store_identity(&self) -> StableStoreIdentity {
        self.discovery.store_identity()
    }

    pub(super) fn selector(&mut self, limit: u64) -> Result<Vec<u8>, Denial> {
        match self.resident.as_deref_mut() {
            Some(resident) => read_record(
                self.discovery,
                resident,
                RecordArtifactFile::CurrentRootSelector,
                limit,
                Denial::MissingSelector,
            ),
            None => self
                .discovery
                .read_current_selector(limit)
                .map_err(Denial::Discovery)?
                .into_bytes()
                .ok_or(Denial::MissingSelector),
        }
    }

    pub(super) fn root(&mut self, generation: u64, limit: u64) -> Result<Vec<u8>, Denial> {
        match self.resident.as_deref_mut() {
            Some(resident) => read_record(
                self.discovery,
                resident,
                RecordArtifactFile::RootManifest { generation },
                limit,
                Denial::MissingRoot,
            ),
            None => self
                .discovery
                .read_root_manifest(generation, limit)
                .map_err(Denial::Discovery)?
                .into_bytes()
                .ok_or(Denial::MissingRoot),
        }
    }

    pub(super) fn checkpoint(&mut self, limit: u64) -> Result<Vec<u8>, Denial> {
        let Some(resident) = self.resident.as_deref_mut() else {
            return self
                .discovery
                .read_current_checkpoint(u64::MAX)
                .map_err(Denial::Discovery)?
                .into_bytes()
                .ok_or(Denial::MissingCheckpoint);
        };
        let mut charged = 0;
        let result = self
            .discovery
            .read_current_checkpoint_with_allocator(limit, |length| {
                let bytes = resident.reserve_bytes(length)?;
                charged = resident.vector_bytes(&bytes)?;
                Ok(bytes)
            });
        let observed = match result {
            Ok(observed) => observed,
            Err(error) => {
                resident.release(charged);
                return Err(discovery_allocation_denial(error));
            }
        };
        observed.into_bytes().ok_or(Denial::MissingCheckpoint)
    }

    pub(super) fn inspect_checkpoint(
        &mut self,
        bytes: &[u8],
        expected: worth_store_physical_format::PhysicalCheckpointIdentity,
        frames: &[Box<[u8]>],
    ) -> Result<
        (
            worth_store_physical_format::PhysicalCheckpointSource,
            Vec<worth_store_physical_format::ReleaseCheckpointCertificateV1>,
            u16,
            u32,
        ),
        Denial,
    > {
        match self.resident.as_deref_mut() {
            Some(resident) => certificate_stream::inspect_checkpoint_with_resident(
                bytes, expected, frames, resident,
            ),
            None => certificate_stream::inspect_checkpoint(bytes, expected, frames),
        }
    }

    pub(super) fn clone_bytes(&mut self, bytes: &[u8]) -> Result<Vec<u8>, Denial> {
        let Some(resident) = self.resident.as_deref_mut() else {
            return Ok(bytes.to_vec());
        };
        let mut copy = resident
            .reserve_bytes(bytes.len())
            .map_err(Denial::Resident)?;
        copy.copy_from_slice(bytes);
        Ok(copy)
    }

    pub(super) fn matches_canonical_root(
        &mut self,
        root: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        bytes: &[u8],
    ) -> Result<bool, Denial> {
        if let Some(resident) = self.resident.as_deref_mut() {
            resident
                .transient(DurablePhysicalRootManifest::maximum_encoding_scratch_bytes() as u64)
                .map_err(Denial::Resident)?;
        }
        Ok(root.encode(format) == bytes)
    }
}

fn read_record(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    resident: &mut StoreRejoinResidentLedger,
    address: RecordArtifactFile,
    limit: u64,
    missing: Denial,
) -> Result<Vec<u8>, Denial> {
    let mut charged = 0;
    let result = discovery.read_record_artifact_with_allocator(address, limit, |length| {
        let bytes = resident.reserve_bytes(length)?;
        charged = resident.vector_bytes(&bytes)?;
        Ok(bytes)
    });
    let observed = match result {
        Ok(observed) => observed,
        Err(error) => {
            resident.release(charged);
            return Err(discovery_allocation_denial(error));
        }
    };
    observed.into_bytes().ok_or(missing)
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn observe_v2_with_resident(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    expected_store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    claim: &VerifiedSelectedReleaseHeadCustodyV2,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<(ObservedRootCheckpoint, ObservedCheckpointSourceRoot), Denial> {
    let mut reader = RootCheckpointReader::resident(discovery, resident);
    let observed = observe_parts(
        &mut reader,
        expected_store,
        format,
        claim.selected_root(),
        checkpoint,
    )?;
    let source = observed.matches_v2_claim(&mut reader, claim)?;
    Ok((observed, source))
}

impl ObservedRootCheckpoint {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_resident_bytes(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Denial> {
        let certificates = resident
            .vector_bytes(&self.release_certificates)
            .map_err(Denial::Resident)?;
        let Self {
            selector_bytes,
            root_bytes,
            checkpoint_bytes,
            release_certificates,
            ..
        } = self;
        drop(release_certificates);
        resident.release(certificates);
        Ok((selector_bytes, root_bytes, checkpoint_bytes))
    }
}

impl ObservedCheckpointSourceRoot {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_bytes(
        self,
    ) -> Vec<u8> {
        self.bytes
    }
}
