//! Bank-owned closed copy of Query canonical-work counters.

use worth_query_host::facade::domain::{
    WorthQueryCanonicalWorkEvidence, WorthQueryCanonicalWorkPhases,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BankCommitCanonicalWorkEvidence {
    basis_preparations: u32,
    digest_derivations: u32,
    canonical_entries: u32,
    canonical_encoded_bytes: usize,
    canonical_material_allocation_bytes: usize,
    sha256_input_bytes: usize,
    sha256_compression_blocks: usize,
    digest_text_materializations: u32,
}

impl BankCommitCanonicalWorkEvidence {
    pub(crate) const fn from_query(work: WorthQueryCanonicalWorkEvidence) -> Self {
        Self {
            basis_preparations: work.basis_preparations(),
            digest_derivations: work.digest_derivations(),
            canonical_entries: work.canonical_entries(),
            canonical_encoded_bytes: work.canonical_encoded_bytes(),
            canonical_material_allocation_bytes: work.canonical_material_allocation_bytes(),
            sha256_input_bytes: work.sha256_input_bytes(),
            sha256_compression_blocks: work.sha256_compression_blocks(),
            digest_text_materializations: work.digest_text_materializations(),
        }
    }

    pub const fn basis_preparations(self) -> u32 {
        self.basis_preparations
    }

    pub const fn digest_derivations(self) -> u32 {
        self.digest_derivations
    }

    pub const fn canonical_entries(self) -> u32 {
        self.canonical_entries
    }

    pub const fn canonical_encoded_bytes(self) -> usize {
        self.canonical_encoded_bytes
    }

    pub const fn canonical_material_allocation_bytes(self) -> usize {
        self.canonical_material_allocation_bytes
    }

    pub const fn sha256_input_bytes(self) -> usize {
        self.sha256_input_bytes
    }

    pub const fn sha256_compression_blocks(self) -> usize {
        self.sha256_compression_blocks
    }

    pub const fn digest_text_materializations(self) -> u32 {
        self.digest_text_materializations
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankCommitCanonicalWorkPhases {
    installation: BankCommitCanonicalWorkEvidence,
    admission: BankCommitCanonicalWorkEvidence,
    execution: BankCommitCanonicalWorkEvidence,
    provider_commit: BankCommitCanonicalWorkEvidence,
    projection: BankCommitCanonicalWorkEvidence,
    live_delivery: BankCommitCanonicalWorkEvidence,
    retry_resolution: BankCommitCanonicalWorkEvidence,
    recovery_inspection: BankCommitCanonicalWorkEvidence,
    publication: BankCommitCanonicalWorkEvidence,
    external_dispatch: BankCommitCanonicalWorkEvidence,
    undo_admission: BankCommitCanonicalWorkEvidence,
    redo_admission: BankCommitCanonicalWorkEvidence,
}

impl BankCommitCanonicalWorkPhases {
    pub(crate) const fn from_query(work: WorthQueryCanonicalWorkPhases) -> Self {
        Self {
            installation: BankCommitCanonicalWorkEvidence::from_query(work.installation()),
            admission: BankCommitCanonicalWorkEvidence::from_query(work.admission()),
            execution: BankCommitCanonicalWorkEvidence::from_query(work.execution()),
            provider_commit: BankCommitCanonicalWorkEvidence::from_query(work.provider_commit()),
            projection: BankCommitCanonicalWorkEvidence::from_query(work.projection()),
            live_delivery: BankCommitCanonicalWorkEvidence::from_query(work.live_delivery()),
            retry_resolution: BankCommitCanonicalWorkEvidence::from_query(work.retry_resolution()),
            recovery_inspection: BankCommitCanonicalWorkEvidence::from_query(
                work.recovery_inspection(),
            ),
            publication: BankCommitCanonicalWorkEvidence::from_query(work.publication()),
            external_dispatch: BankCommitCanonicalWorkEvidence::from_query(
                work.external_dispatch(),
            ),
            undo_admission: BankCommitCanonicalWorkEvidence::from_query(work.undo_admission()),
            redo_admission: BankCommitCanonicalWorkEvidence::from_query(work.redo_admission()),
        }
    }

    pub const fn installation(self) -> BankCommitCanonicalWorkEvidence {
        self.installation
    }
    pub const fn admission(self) -> BankCommitCanonicalWorkEvidence {
        self.admission
    }
    pub const fn execution(self) -> BankCommitCanonicalWorkEvidence {
        self.execution
    }
    pub const fn provider_commit(self) -> BankCommitCanonicalWorkEvidence {
        self.provider_commit
    }
    pub const fn projection(self) -> BankCommitCanonicalWorkEvidence {
        self.projection
    }
    pub const fn live_delivery(self) -> BankCommitCanonicalWorkEvidence {
        self.live_delivery
    }
    pub const fn retry_resolution(self) -> BankCommitCanonicalWorkEvidence {
        self.retry_resolution
    }
    pub const fn recovery_inspection(self) -> BankCommitCanonicalWorkEvidence {
        self.recovery_inspection
    }
    pub const fn publication(self) -> BankCommitCanonicalWorkEvidence {
        self.publication
    }
    pub const fn external_dispatch(self) -> BankCommitCanonicalWorkEvidence {
        self.external_dispatch
    }
    pub const fn undo_admission(self) -> BankCommitCanonicalWorkEvidence {
        self.undo_admission
    }
    pub const fn redo_admission(self) -> BankCommitCanonicalWorkEvidence {
        self.redo_admission
    }
}
