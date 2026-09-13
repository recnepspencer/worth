use std::io::Write;

use sha2::{Digest, Sha256};

use crate::{PageGenerationCell, PhysicalReference, PhysicalReferenceKind, PhysicalRootReference};

pub(super) const CHECKPOINT_BACKUP_MAGIC: &[u8; 8] = b"WORTHCKP";
pub(super) const CHECKPOINT_BACKUP_VERSION: u16 = 1;
pub(super) const CHECKPOINT_BACKUP_HEADER_BYTES: usize = 78;
pub(super) const CHECKPOINT_BACKUP_PAGE_ROW_BYTES: usize = 32;
pub(super) const CHECKPOINT_BACKUP_FOOTER_BYTES: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointBackupArtifactInput {
    pub checkpoint_identity: String,
    pub manifest_generation: u64,
    pub durable_checkpoint_lsn: u64,
    pub root: PhysicalReference,
    pub covered_lsn: (u64, u64),
    pub redo_lsn: u64,
    pub pages: Vec<(PageGenerationCell, u64)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointBackupArtifact {
    checkpoint_identity: String,
    manifest_generation: u64,
    durable_checkpoint_lsn: u64,
    root: PhysicalReference,
    covered_lsn: (u64, u64),
    redo_lsn: u64,
    pages: Vec<(PageGenerationCell, u64)>,
}

impl CheckpointBackupArtifact {
    pub fn from_input(mut input: CheckpointBackupArtifactInput) -> Option<Self> {
        if input.checkpoint_identity.trim().is_empty()
            || input.manifest_generation == 0
            || input.root.kind() != PhysicalReferenceKind::RootPublication
            || input.covered_lsn.0 >= input.covered_lsn.1
            || input.redo_lsn < input.covered_lsn.0
            || input.redo_lsn >= input.covered_lsn.1
            || input.durable_checkpoint_lsn < input.redo_lsn
            || input.durable_checkpoint_lsn > input.covered_lsn.1
            || input.pages.is_empty()
        {
            return None;
        }
        input.pages.sort_by_key(|(page, _)| {
            (
                page.segment_id().get(),
                page.page_id().get(),
                page.generation().get(),
            )
        });
        let valid_pages = input.pages.windows(2).all(|pair| pair[0].0 != pair[1].0)
            && input
                .pages
                .iter()
                .all(|(_, page_lsn)| *page_lsn >= input.redo_lsn);
        if !valid_pages {
            return None;
        }
        Some(Self {
            checkpoint_identity: input.checkpoint_identity,
            manifest_generation: input.manifest_generation,
            durable_checkpoint_lsn: input.durable_checkpoint_lsn,
            root: input.root,
            covered_lsn: input.covered_lsn,
            redo_lsn: input.redo_lsn,
            pages: input.pages,
        })
    }

    pub fn checkpoint_identity(&self) -> &str {
        &self.checkpoint_identity
    }

    pub const fn manifest_generation(&self) -> u64 {
        self.manifest_generation
    }

    pub const fn durable_checkpoint_lsn(&self) -> u64 {
        self.durable_checkpoint_lsn
    }

    pub const fn root(&self) -> PhysicalReference {
        self.root
    }

    pub const fn root_reference(&self) -> PhysicalRootReference {
        self.root
            .root_reference()
            .expect("checkpoint artifact root is validated")
    }

    pub const fn root_generation(&self) -> u64 {
        self.root.generation().get()
    }

    pub const fn covered_lsn(&self) -> (u64, u64) {
        self.covered_lsn
    }

    pub const fn redo_lsn(&self) -> u64 {
        self.redo_lsn
    }

    pub fn pages(&self) -> &[(PageGenerationCell, u64)] {
        &self.pages
    }

    pub fn encode(&self, mut output: impl Write) -> Result<u64, std::io::Error> {
        let mut digest = Sha256::new();
        let identity = self.checkpoint_identity.as_bytes();
        let header = encode_header(self, identity.len())?;
        write_hashed(&mut output, &mut digest, &header)?;
        for (page, page_lsn) in &self.pages {
            let row = encode_page_row(*page, *page_lsn);
            write_hashed(&mut output, &mut digest, &row)?;
        }
        write_hashed(&mut output, &mut digest, identity)?;
        output.write_all(&digest.finalize())?;
        Ok(CHECKPOINT_BACKUP_HEADER_BYTES as u64
            + self.pages.len() as u64 * CHECKPOINT_BACKUP_PAGE_ROW_BYTES as u64
            + identity.len() as u64
            + CHECKPOINT_BACKUP_FOOTER_BYTES as u64)
    }
}

fn encode_header(
    artifact: &CheckpointBackupArtifact,
    identity_bytes: usize,
) -> Result<[u8; CHECKPOINT_BACKUP_HEADER_BYTES], std::io::Error> {
    let mut header = [0_u8; CHECKPOINT_BACKUP_HEADER_BYTES];
    header[0..8].copy_from_slice(CHECKPOINT_BACKUP_MAGIC);
    header[8..10].copy_from_slice(&CHECKPOINT_BACKUP_VERSION.to_le_bytes());
    let values = [
        artifact.manifest_generation,
        artifact.durable_checkpoint_lsn,
        artifact.root_reference().get(),
        artifact.root_generation(),
        artifact.covered_lsn.0,
        artifact.covered_lsn.1,
        artifact.redo_lsn,
        artifact.pages.len() as u64,
    ];
    for (index, value) in values.into_iter().enumerate() {
        let offset = 10 + index * 8;
        header[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
    let identity_bytes = u32::try_from(identity_bytes)
        .map_err(|_| std::io::Error::other("checkpoint identity too large"))?;
    header[74..78].copy_from_slice(&identity_bytes.to_le_bytes());
    Ok(header)
}

fn encode_page_row(
    page: PageGenerationCell,
    page_lsn: u64,
) -> [u8; CHECKPOINT_BACKUP_PAGE_ROW_BYTES] {
    let mut row = [0_u8; CHECKPOINT_BACKUP_PAGE_ROW_BYTES];
    row[0..8].copy_from_slice(&page.segment_id().get().to_le_bytes());
    row[8..16].copy_from_slice(&page.page_id().get().to_le_bytes());
    row[16..24].copy_from_slice(&page.generation().get().to_le_bytes());
    row[24..32].copy_from_slice(&page_lsn.to_le_bytes());
    row
}

fn write_hashed(
    output: &mut impl Write,
    digest: &mut Sha256,
    bytes: &[u8],
) -> Result<(), std::io::Error> {
    output.write_all(bytes)?;
    digest.update(bytes);
    Ok(())
}
