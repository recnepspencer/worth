use crate::validation::{
    IntegrityValidatedCheckpointBinding, IntegrityValidatedCheckpointBindingCompaction,
    IntegrityValidatedCheckpointDirtyBasis, IntegrityValidatedCheckpointFooter,
    IntegrityValidatedCheckpointStreamHeader, UntrustedPhysicalArtifact,
};
use crate::{PhysicalByteRange, PhysicalIntegrityRejection};

mod assembly;
mod facts;

pub use assembly::ValidatedCheckpointStreamAssembly;
pub use facts::{VerifiedCheckpointCompactionCutover, VerifiedCheckpointFacts};

#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedCheckpointStream {
    facts: VerifiedCheckpointFacts,
    certificate_records: Box<[Box<[u8]>]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifiedCheckpointStreamAssemblyDenial {
    SourceIdentityMismatch,
    RecordScopeMismatch,
    InputIncarnationMismatch,
    FooterBasisMismatch(PhysicalIntegrityRejection),
}

impl VerifiedCheckpointStream {
    /// Certificate backing only; inline facts own no heap storage.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.certificate_records.iter().try_fold(
            u64::try_from(std::mem::size_of_val(&*self.certificate_records)).ok()?,
            |bytes, frame| bytes.checked_add(u64::try_from(frame.len()).ok()?),
        )
    }

    pub fn assemble_from_validated_records<'records, 'media>(
        complete_stream: UntrustedPhysicalArtifact<'media>,
        header: &'records IntegrityValidatedCheckpointStreamHeader<'media>,
        dirty: &'records [&'records IntegrityValidatedCheckpointDirtyBasis<'media>],
        compaction: &'records IntegrityValidatedCheckpointBindingCompaction<'media>,
        bindings: &'records [&'records IntegrityValidatedCheckpointBinding<'media>],
        footer: &'records IntegrityValidatedCheckpointFooter<'media>,
    ) -> Result<Self, VerifiedCheckpointStreamAssemblyDenial> {
        Self::assemble_from_validated_records_with_certificates(
            complete_stream,
            header,
            dirty,
            compaction,
            bindings,
            &[],
            footer,
        )
    }

    /// Pure assembly convenience. Managed runtimes use allocation-free preflight
    /// and retain prepared, already-funded certificate storage instead.
    pub fn assemble_from_validated_records_with_certificates<'records, 'media>(
        complete_stream: UntrustedPhysicalArtifact<'media>,
        header: &'records IntegrityValidatedCheckpointStreamHeader<'media>,
        dirty: &'records [&'records IntegrityValidatedCheckpointDirtyBasis<'media>],
        compaction: &'records IntegrityValidatedCheckpointBindingCompaction<'media>,
        bindings: &'records [&'records IntegrityValidatedCheckpointBinding<'media>],
        certificates: &'records [(PhysicalByteRange, &'media [u8])],
        footer: &'records IntegrityValidatedCheckpointFooter<'media>,
    ) -> Result<Self, VerifiedCheckpointStreamAssemblyDenial> {
        let assembly = Self::validate_records_with_certificates(
            complete_stream,
            header,
            dirty,
            compaction,
            bindings,
            certificates,
            footer,
        )?;
        let prepared = certificates
            .iter()
            .map(|(_, bytes)| bytes.to_vec().into_boxed_slice())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        assembly.retain_prepared_certificates(prepared)
    }

    /// Validates all records and the aggregate without allocating retained backing.
    pub fn validate_records_with_certificates<'records, 'media>(
        complete_stream: UntrustedPhysicalArtifact<'media>,
        header: &'records IntegrityValidatedCheckpointStreamHeader<'media>,
        dirty: &'records [&'records IntegrityValidatedCheckpointDirtyBasis<'media>],
        compaction: &'records IntegrityValidatedCheckpointBindingCompaction<'media>,
        bindings: &'records [&'records IntegrityValidatedCheckpointBinding<'media>],
        certificates: &'records [(PhysicalByteRange, &'media [u8])],
        footer: &'records IntegrityValidatedCheckpointFooter<'media>,
    ) -> Result<
        ValidatedCheckpointStreamAssembly<'records, 'media>,
        VerifiedCheckpointStreamAssemblyDenial,
    > {
        assembly::validate(
            complete_stream,
            header,
            dirty,
            compaction,
            bindings,
            certificates,
            footer,
        )
    }

    pub const fn facts(&self) -> VerifiedCheckpointFacts {
        self.facts
    }

    pub const fn source(&self) -> worth_store_physical_format::PhysicalCheckpointSource {
        self.facts.source()
    }

    pub const fn footer(&self) -> worth_store_physical_format::CheckpointStreamFooter {
        self.facts.footer()
    }

    pub const fn encoded_bytes(&self) -> u64 {
        self.facts.encoded_bytes()
    }

    pub const fn encoded_digest(&self) -> [u8; 32] {
        self.facts.encoded_digest()
    }

    pub const fn compaction_cutover(&self) -> VerifiedCheckpointCompactionCutover {
        self.facts.compaction_cutover()
    }

    pub fn certificate_records(&self) -> &[Box<[u8]>] {
        &self.certificate_records
    }
}
