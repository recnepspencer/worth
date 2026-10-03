use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    checkpoint_stream_encoded_digest, maximum_current_root_entries,
    store_namespace::StableStoreIdentity, DurablePhysicalRootManifest, DurableRootSelector,
    PhysicalCheckpointSource, PhysicalRecordFormatDeclaration, ReleaseCheckpointCertificateV1,
    RootSelectorRole, ROOT_SELECTOR_BYTES,
};
use worth_store_physical_integrity::{VerifiedCheckpointFacts, VerifiedCheckpointStream};
use worth_store_recovery_physics::{
    VerifiedSelectedCheckpointCustody, VerifiedSelectedReleaseHeadCustodyV2,
};

use super::SelectedMediaRejoinDenial as Denial;

#[path = "root_checkpoint/addressed.rs"]
pub(super) mod addressed;
#[path = "root_checkpoint/certificate_stream.rs"]
mod certificate_stream;
pub(super) use certificate_stream::inspect_checkpoint;
#[path = "root_checkpoint/resident_observation.rs"]
mod resident_observation;
pub(super) use resident_observation::observe_v2_with_resident;
use resident_observation::RootCheckpointReader;

/// Actual bytes observed through one admitted discovery, never a token-supplied route.
pub(super) struct ObservedRootCheckpoint {
    selector: DurableRootSelector,
    selector_bytes: Vec<u8>,
    root: DurablePhysicalRootManifest,
    root_bytes: Vec<u8>,
    root_sha256: [u8; 32],
    checkpoint_bytes: Vec<u8>,
    checkpoint_source: PhysicalCheckpointSource,
    release_certificates: Vec<ReleaseCheckpointCertificateV1>,
    release_record_count: u16,
    release_encoded_bytes: u32,
}

/// Canonical checkpoint-source root bytes read by Store, not copied from a claim.
pub(super) struct ObservedCheckpointSourceRoot {
    root: DurablePhysicalRootManifest,
    bytes: Vec<u8>,
}

impl ObservedCheckpointSourceRoot {
    pub(super) fn root(&self) -> &DurablePhysicalRootManifest {
        &self.root
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl ObservedRootCheckpoint {
    pub(super) const fn selector(&self) -> DurableRootSelector {
        self.selector
    }
    pub(super) fn selector_bytes(&self) -> &[u8] {
        &self.selector_bytes
    }
    pub(super) fn root(&self) -> &DurablePhysicalRootManifest {
        &self.root
    }
    pub(super) fn root_bytes(&self) -> &[u8] {
        &self.root_bytes
    }

    /// A public recovery claim is only a transcript. Its checkpoint must be
    /// byte-identical to the Store-owned observation, with exactly the selected
    /// release accumulator and no extra release record.
    pub(super) fn matches_claim(
        &self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        claim: &VerifiedSelectedCheckpointCustody,
    ) -> Result<(), Denial> {
        let mut expected_releases = claim
            .batches()
            .iter()
            .copied()
            .map(ReleaseCheckpointCertificateV1::Batch)
            .collect::<Vec<_>>();
        expected_releases.push(ReleaseCheckpointCertificateV1::Accumulator(
            claim.accumulator(),
        ));
        self.matches_parts(
            &mut RootCheckpointReader::legacy(discovery),
            claim.selected_root(),
            claim.selected_root_sha256(),
            claim.checkpoint(),
            claim.source_root_sha256(),
            self.release_certificates == expected_releases,
            claim.release_certificate_record_count(),
            claim.release_certificate_encoded_bytes(),
        )
        .map(|_| ())
    }

    fn matches_v2_claim(
        &self,
        reader: &mut RootCheckpointReader<'_, '_>,
        claim: &VerifiedSelectedReleaseHeadCustodyV2,
    ) -> Result<ObservedCheckpointSourceRoot, Denial> {
        let releases_match = self.release_certificates.len() == claim.batches().len() + 1
            && self.release_certificates[..claim.batches().len()]
                .iter()
                .zip(claim.batches())
                .all(|(observed, expected)| {
                    *observed == ReleaseCheckpointCertificateV1::Batch(*expected)
                })
            && self.release_certificates.last()
                == Some(&ReleaseCheckpointCertificateV1::AccumulatorV2(
                    claim.accumulator_v2(),
                ));
        let source = self.matches_parts(
            reader,
            claim.selected_root(),
            claim.selected_root_sha256(),
            claim.checkpoint(),
            claim.source_root_sha256(),
            releases_match,
            claim.release_certificate_record_count(),
            claim.release_certificate_encoded_bytes(),
        )?;
        if source.root() != claim.checkpoint_source_root() {
            return Err(Denial::RootBinding);
        }
        Ok(source)
    }

    #[allow(clippy::too_many_arguments)]
    fn matches_parts(
        &self,
        reader: &mut RootCheckpointReader<'_, '_>,
        selected_root: &DurablePhysicalRootManifest,
        selected_root_sha256: [u8; 32],
        checkpoint: &VerifiedCheckpointFacts,
        source_root_sha256: [u8; 32],
        releases_match: bool,
        release_record_count: u16,
        release_encoded_bytes: u32,
    ) -> Result<ObservedCheckpointSourceRoot, Denial> {
        if self.root != *selected_root
            || self.root_sha256 != selected_root_sha256
            || self.checkpoint_source != checkpoint.source()
            || self.checkpoint_bytes.len() as u64 != checkpoint.encoded_bytes()
            || checkpoint_stream_encoded_digest(&self.checkpoint_bytes)
                != checkpoint.encoded_digest()
            || !releases_match
            || self.release_record_count != release_record_count
            || self.release_encoded_bytes != release_encoded_bytes
        {
            return Err(Denial::CheckpointBinding);
        }
        let source_basis = self.checkpoint_source.root();
        let source = if source_basis.generation() == self.root.generation() {
            if self.root.tree_identity() != source_basis.tree_identity() {
                return Err(Denial::RootBinding);
            }
            ObservedCheckpointSourceRoot {
                root: self.root.clone(),
                bytes: reader.clone_bytes(&self.root_bytes)?,
            }
        } else {
            let bytes = reader.root(
                source_basis.generation(),
                u64::from(self.selector.format().page_size().bytes()),
            )?;
            let (source_root, source_format) = DurablePhysicalRootManifest::decode(
                &bytes,
                maximum_current_root_entries(self.selector.format()),
            )
            .map_err(|_| Denial::RootBinding)?;
            if source_format != self.selector.format()
                || source_root.generation() != source_basis.generation()
                || source_root.tree_identity() != source_basis.tree_identity()
                || !reader.matches_canonical_root(&source_root, source_format, &bytes)?
            {
                return Err(Denial::RootBinding);
            }
            ObservedCheckpointSourceRoot {
                root: source_root,
                bytes,
            }
        };
        if <[u8; 32]>::from(Sha256::digest(source.bytes())) != source_root_sha256 {
            return Err(Denial::RootBinding);
        }
        Ok(source)
    }
}

fn observe_parts(
    reader: &mut RootCheckpointReader<'_, '_>,
    expected_store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    expected_root: &DurablePhysicalRootManifest,
    verified_checkpoint: &VerifiedCheckpointStream,
) -> Result<ObservedRootCheckpoint, Denial> {
    let expected_checkpoint = verified_checkpoint.source().identity();
    if reader.store_identity() != expected_store
        || expected_checkpoint.store_identity() != expected_store
    {
        return Err(Denial::RootBinding);
    }
    let selector_bytes = reader.selector(ROOT_SELECTOR_BYTES as u64)?;
    let selector = DurableRootSelector::decode(&selector_bytes).map_err(|_| Denial::RootBinding)?;
    if selector.encode() != selector_bytes.as_slice()
        || selector.store_identity() != expected_store
        || selector.format() != format
        || selector.role() != RootSelectorRole::Current
        || selector.root_generation() != expected_root.generation()
    {
        return Err(Denial::RootBinding);
    }
    let root_bytes = reader.root(
        selector.root_generation(),
        format.page_size().bytes() as u64,
    )?;
    let (root, decoded_format) =
        DurablePhysicalRootManifest::decode(&root_bytes, expected_root.node_capacity())
            .map_err(|_| Denial::RootBinding)?;
    if decoded_format != format
        || root != *expected_root
        || root.generation() != selector.root_generation()
        || !reader.matches_canonical_root(&root, format, &root_bytes)?
    {
        return Err(Denial::RootBinding);
    }
    let root_sha256 = Sha256::digest(&root_bytes).into();
    // Discovery's overall admission budget is the checkpoint read bound.
    let checkpoint_bytes = reader.checkpoint(verified_checkpoint.encoded_bytes())?;
    if checkpoint_bytes.len() as u64 != verified_checkpoint.encoded_bytes()
        || checkpoint_stream_encoded_digest(&checkpoint_bytes)
            != verified_checkpoint.encoded_digest()
    {
        return Err(Denial::CheckpointBinding);
    }
    let (checkpoint_source, release_certificates, release_record_count, release_encoded_bytes) =
        reader.inspect_checkpoint(
            &checkpoint_bytes,
            expected_checkpoint,
            verified_checkpoint.certificate_records(),
        )?;
    let observed = ObservedRootCheckpoint {
        selector,
        selector_bytes,
        root,
        root_bytes,
        root_sha256,
        checkpoint_bytes,
        checkpoint_source,
        release_certificates,
        release_record_count,
        release_encoded_bytes,
    };
    Ok(observed)
}
