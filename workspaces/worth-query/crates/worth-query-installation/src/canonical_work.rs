use worth_foundational::facade::CanonicalDigestWorkEvidence;

// Complete application installation includes package and schema meaning. Keep
// this finite while admitting the measured House composition that first
// crossed the former 4 MiB ceiling through ordinary typed declarations.
pub(crate) const INSTALLATION_MAXIMUM_CANONICAL_BYTES: usize = 8 * 1_024 * 1_024;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryCanonicalWorkEvidence {
    basis_preparations: u32,
    digest_derivations: u32,
    canonical_entries: u32,
    canonical_encoded_bytes: usize,
    canonical_material_allocation_bytes: usize,
    sha256_input_bytes: usize,
    sha256_compression_blocks: usize,
    digest_text_materializations: u32,
}

impl WorthQueryCanonicalWorkEvidence {
    pub const fn zero() -> Self {
        Self {
            basis_preparations: 0,
            digest_derivations: 0,
            canonical_entries: 0,
            canonical_encoded_bytes: 0,
            canonical_material_allocation_bytes: 0,
            sha256_input_bytes: 0,
            sha256_compression_blocks: 0,
            digest_text_materializations: 0,
        }
    }

    pub const fn one_digest(work: CanonicalDigestWorkEvidence) -> Self {
        Self {
            basis_preparations: 1,
            digest_derivations: 1,
            canonical_entries: work.canonical_entry_count(),
            canonical_encoded_bytes: work.canonical_encoded_bytes(),
            canonical_material_allocation_bytes: work.canonical_material_allocation_bytes(),
            sha256_input_bytes: work.sha256_input_bytes(),
            sha256_compression_blocks: work.sha256_compression_block_count(),
            digest_text_materializations: 0,
        }
    }

    pub(crate) const fn one_basis_preparation(
        canonical_entries: u32,
        canonical_encoded_bytes: usize,
    ) -> Self {
        Self {
            basis_preparations: 1,
            digest_derivations: 0,
            canonical_entries,
            canonical_encoded_bytes,
            canonical_material_allocation_bytes: canonical_encoded_bytes,
            sha256_input_bytes: 0,
            sha256_compression_blocks: 0,
            digest_text_materializations: 0,
        }
    }

    pub const fn combine(self, other: Self) -> Self {
        Self {
            basis_preparations: self
                .basis_preparations
                .saturating_add(other.basis_preparations),
            digest_derivations: self
                .digest_derivations
                .saturating_add(other.digest_derivations),
            canonical_entries: self
                .canonical_entries
                .saturating_add(other.canonical_entries),
            canonical_encoded_bytes: self
                .canonical_encoded_bytes
                .saturating_add(other.canonical_encoded_bytes),
            canonical_material_allocation_bytes: self
                .canonical_material_allocation_bytes
                .saturating_add(other.canonical_material_allocation_bytes),
            sha256_input_bytes: self
                .sha256_input_bytes
                .saturating_add(other.sha256_input_bytes),
            sha256_compression_blocks: self
                .sha256_compression_blocks
                .saturating_add(other.sha256_compression_blocks),
            digest_text_materializations: self
                .digest_text_materializations
                .saturating_add(other.digest_text_materializations),
        }
    }

    /// Record an explicit diagnostic, publication, provider-wire, or support
    /// rendering of an already-derived fixed-width digest.
    pub const fn with_digest_text_materializations(self, count: u32) -> Self {
        Self {
            basis_preparations: self.basis_preparations,
            digest_derivations: self.digest_derivations,
            canonical_entries: self.canonical_entries,
            canonical_encoded_bytes: self.canonical_encoded_bytes,
            canonical_material_allocation_bytes: self.canonical_material_allocation_bytes,
            sha256_input_bytes: self.sha256_input_bytes,
            sha256_compression_blocks: self.sha256_compression_blocks,
            digest_text_materializations: self.digest_text_materializations.saturating_add(count),
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
pub struct WorthQueryCanonicalWorkPhases {
    installation: WorthQueryCanonicalWorkEvidence,
    admission: WorthQueryCanonicalWorkEvidence,
    execution: WorthQueryCanonicalWorkEvidence,
    provider_commit: WorthQueryCanonicalWorkEvidence,
    projection: WorthQueryCanonicalWorkEvidence,
    live_delivery: WorthQueryCanonicalWorkEvidence,
    retry_resolution: WorthQueryCanonicalWorkEvidence,
    recovery_inspection: WorthQueryCanonicalWorkEvidence,
    publication: WorthQueryCanonicalWorkEvidence,
    external_dispatch: WorthQueryCanonicalWorkEvidence,
    undo_admission: WorthQueryCanonicalWorkEvidence,
    redo_admission: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryCanonicalWorkPhases {
    /// Construct phase evidence. The three Phase-8 slots must be supplied at
    /// every construction site (arch law 9 / R8.13); they are never invented.
    pub const fn new(
        installation: WorthQueryCanonicalWorkEvidence,
        admission: WorthQueryCanonicalWorkEvidence,
        external_dispatch: WorthQueryCanonicalWorkEvidence,
        undo_admission: WorthQueryCanonicalWorkEvidence,
        redo_admission: WorthQueryCanonicalWorkEvidence,
    ) -> Self {
        Self {
            installation,
            admission,
            execution: WorthQueryCanonicalWorkEvidence::zero(),
            provider_commit: WorthQueryCanonicalWorkEvidence::zero(),
            projection: WorthQueryCanonicalWorkEvidence::zero(),
            live_delivery: WorthQueryCanonicalWorkEvidence::zero(),
            retry_resolution: WorthQueryCanonicalWorkEvidence::zero(),
            recovery_inspection: WorthQueryCanonicalWorkEvidence::zero(),
            publication: WorthQueryCanonicalWorkEvidence::zero(),
            external_dispatch,
            undo_admission,
            redo_admission,
        }
    }

    pub const fn installation(self) -> WorthQueryCanonicalWorkEvidence {
        self.installation
    }

    pub const fn admission(self) -> WorthQueryCanonicalWorkEvidence {
        self.admission
    }

    pub const fn execution(self) -> WorthQueryCanonicalWorkEvidence {
        self.execution
    }

    pub const fn with_execution_work(self, work: WorthQueryCanonicalWorkEvidence) -> Self {
        Self {
            installation: self.installation,
            admission: self.admission,
            execution: self.execution.combine(work),
            provider_commit: self.provider_commit,
            projection: self.projection,
            live_delivery: self.live_delivery,
            retry_resolution: self.retry_resolution,
            recovery_inspection: self.recovery_inspection,
            publication: self.publication,
            external_dispatch: self.external_dispatch,
            undo_admission: self.undo_admission,
            redo_admission: self.redo_admission,
        }
    }

    pub const fn with_external_dispatch_work(self, work: WorthQueryCanonicalWorkEvidence) -> Self {
        Self {
            installation: self.installation,
            admission: self.admission,
            execution: self.execution,
            provider_commit: self.provider_commit,
            projection: self.projection,
            live_delivery: self.live_delivery,
            retry_resolution: self.retry_resolution,
            recovery_inspection: self.recovery_inspection,
            publication: self.publication,
            external_dispatch: self.external_dispatch.combine(work),
            undo_admission: self.undo_admission,
            redo_admission: self.redo_admission,
        }
    }

    pub const fn provider_commit(self) -> WorthQueryCanonicalWorkEvidence {
        self.provider_commit
    }

    pub const fn projection(self) -> WorthQueryCanonicalWorkEvidence {
        self.projection
    }

    pub const fn live_delivery(self) -> WorthQueryCanonicalWorkEvidence {
        self.live_delivery
    }

    pub const fn retry_resolution(self) -> WorthQueryCanonicalWorkEvidence {
        self.retry_resolution
    }

    pub const fn recovery_inspection(self) -> WorthQueryCanonicalWorkEvidence {
        self.recovery_inspection
    }

    pub const fn publication(self) -> WorthQueryCanonicalWorkEvidence {
        self.publication
    }

    pub const fn external_dispatch(self) -> WorthQueryCanonicalWorkEvidence {
        self.external_dispatch
    }

    pub const fn undo_admission(self) -> WorthQueryCanonicalWorkEvidence {
        self.undo_admission
    }

    pub const fn redo_admission(self) -> WorthQueryCanonicalWorkEvidence {
        self.redo_admission
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_eight_slots_are_supplied_at_construction() {
        let phases = WorthQueryCanonicalWorkPhases::new(
            WorthQueryCanonicalWorkEvidence::zero(),
            WorthQueryCanonicalWorkEvidence::zero(),
            WorthQueryCanonicalWorkEvidence::zero(),
            WorthQueryCanonicalWorkEvidence::zero(),
            WorthQueryCanonicalWorkEvidence::zero(),
        );
        assert_eq!(
            phases.external_dispatch(),
            WorthQueryCanonicalWorkEvidence::zero()
        );
        assert_eq!(
            phases.undo_admission(),
            WorthQueryCanonicalWorkEvidence::zero()
        );
        assert_eq!(
            phases.redo_admission(),
            WorthQueryCanonicalWorkEvidence::zero()
        );
    }
}
