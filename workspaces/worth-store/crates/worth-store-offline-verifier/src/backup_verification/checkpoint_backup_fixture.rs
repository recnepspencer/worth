use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PageGenerationCell, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalRootReference, PhysicalSegmentId,
};

use super::super::checkpoint_backup_verification::BoundedCheckpointBackupVerificationRequest;

pub(super) struct RawFixture {
    pub(super) bytes: Vec<u8>,
    pub(super) digest: [u8; 32],
    pub(super) identity: Vec<u8>,
    page_count: usize,
    pages: Vec<(PageGenerationCell, u64)>,
}

impl RawFixture {
    pub(super) fn decoder_allocation_bytes(&self) -> u64 {
        (self.identity.len()
            + self.page_count
                * std::mem::size_of::<(worth_store_physical_format::PageGenerationCell, u64)>())
            as u64
    }

    pub(super) fn peak_buffer_bytes(&self) -> u64 {
        self.decoder_allocation_bytes() + (78 + 32 + 32) as u64
    }
}

pub(super) fn request(
    fixture: &RawFixture,
    max_buffer_bytes: usize,
) -> BoundedCheckpointBackupVerificationRequest<'_> {
    BoundedCheckpointBackupVerificationRequest {
        checkpoint_identity: std::str::from_utf8(&fixture.identity).expect("fixture identity"),
        manifest_generation: 3,
        durable_checkpoint_lsn: 10,
        expected_root: root_cell(1),
        expected_authority_fingerprint: [7; 32],
        expected_frontier_digest: independent_frontier_digest(
            [7; 32],
            std::str::from_utf8(&fixture.identity).expect("fixture identity"),
            3,
            10,
            root_cell(1),
            (1, 11),
            10,
            &fixture.pages,
        ),
        expected_bytes: fixture.bytes.len() as u64,
        expected_digest: fixture.digest,
        max_buffer_bytes,
    }
}

pub(super) fn raw_fixture(root_reference: u64, page_lsn: u64) -> RawFixture {
    raw_fixture_with_pages(root_reference, &[page_lsn])
}

pub(super) fn raw_fixture_with_pages(root_reference: u64, page_lsns: &[u64]) -> RawFixture {
    let identity = b"raw-checkpoint-identity".to_vec();
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let pages: Vec<_> = page_lsns
        .iter()
        .copied()
        .enumerate()
        .map(|(index, page_lsn)| {
            (
                generations
                    .page_cell(
                        PhysicalSegmentId::from_raw(1).expect("segment"),
                        PhysicalPageId::from_raw(index as u64 + 1).expect("page"),
                    )
                    .with_page_generation(PhysicalGeneration::from_raw(1).expect("generation")),
                page_lsn,
            )
        })
        .collect();
    let mut body = vec![0_u8; 78];
    body[0..8].copy_from_slice(b"WORTHCKP");
    body[8..10].copy_from_slice(&1_u16.to_le_bytes());
    put_u64(&mut body, 10, 3);
    put_u64(&mut body, 18, 10);
    put_u64(&mut body, 26, root_reference);
    put_u64(&mut body, 34, 1);
    put_u64(&mut body, 42, 1);
    put_u64(&mut body, 50, 11);
    put_u64(&mut body, 58, 10);
    put_u64(&mut body, 66, page_lsns.len() as u64);
    put_u32(&mut body, 74, identity.len() as u32);
    for (index, page_lsn) in page_lsns.iter().copied().enumerate() {
        body.extend_from_slice(&1_u64.to_le_bytes());
        body.extend_from_slice(&(index as u64 + 1).to_le_bytes());
        body.extend_from_slice(&1_u64.to_le_bytes());
        body.extend_from_slice(&page_lsn.to_le_bytes());
    }
    body.extend_from_slice(&identity);
    let internal_digest: [u8; 32] = Sha256::digest(&body).into();
    body.extend_from_slice(&internal_digest);
    let digest: [u8; 32] = Sha256::digest(&body).into();
    RawFixture {
        bytes: body,
        digest,
        identity,
        page_count: page_lsns.len(),
        pages,
    }
}

pub(super) fn rehashed_u64_fixture(original: &RawFixture, offset: usize, value: u64) -> RawFixture {
    let mut bytes = original.bytes.clone();
    put_u64(&mut bytes, offset, value);
    rehashed_bytes(original, bytes)
}

pub(super) fn rehashed_byte_fixture(original: &RawFixture, offset: usize, value: u8) -> RawFixture {
    let mut bytes = original.bytes.clone();
    bytes[offset] = value;
    rehashed_bytes(original, bytes)
}

pub(super) fn rehashed_page_swap(original: &RawFixture) -> RawFixture {
    let mut bytes = original.bytes.clone();
    let first = 78;
    let second = first + 32;
    for offset in 0..32 {
        bytes.swap(first + offset, second + offset);
    }
    rehashed_bytes(original, bytes)
}

fn rehashed_bytes(original: &RawFixture, mut bytes: Vec<u8>) -> RawFixture {
    let internal_digest_start = bytes.len() - 32;
    let internal_digest: [u8; 32] = Sha256::digest(&bytes[..internal_digest_start]).into();
    bytes[internal_digest_start..].copy_from_slice(&internal_digest);
    let artifact_digest: [u8; 32] = Sha256::digest(&bytes).into();
    RawFixture {
        bytes,
        digest: artifact_digest,
        identity: original.identity.clone(),
        page_count: original.page_count,
        pages: original.pages.clone(),
    }
}

fn independent_frontier_digest(
    authority_fingerprint: [u8; 32],
    checkpoint_identity: &str,
    manifest_generation: u64,
    durable_checkpoint_lsn: u64,
    root: worth_store_physical_format::RootPublicationCell,
    covered_lsn: (u64, u64),
    redo_lsn: u64,
    pages: &[(PageGenerationCell, u64)],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store:checkpoint-backup-frontier:v1\0");
    digest.update(authority_fingerprint);
    digest.update((checkpoint_identity.len() as u64).to_le_bytes());
    digest.update(checkpoint_identity.as_bytes());
    digest.update(manifest_generation.to_le_bytes());
    digest.update(durable_checkpoint_lsn.to_le_bytes());
    digest.update(root.root_reference().get().to_le_bytes());
    digest.update(root.generation().get().to_le_bytes());
    digest.update(covered_lsn.0.to_le_bytes());
    digest.update(covered_lsn.1.to_le_bytes());
    digest.update(redo_lsn.to_le_bytes());
    digest.update((pages.len() as u64).to_le_bytes());
    for (page, page_lsn) in pages {
        digest.update(page.segment_id().get().to_le_bytes());
        digest.update(page.page_id().get().to_le_bytes());
        digest.update(page.generation().get().to_le_bytes());
        digest.update(page_lsn.to_le_bytes());
    }
    digest.finalize().into()
}

fn root_cell(root_reference: u64) -> worth_store_physical_format::RootPublicationCell {
    let reference = PhysicalRootReference::from_raw(root_reference).expect("root reference");
    let generation = PhysicalGeneration::from_raw(1).expect("root generation");
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .root_publication_cell(reference)
        .with_root_publication_generation(generation)
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
