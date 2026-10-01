use worth_store_physical_format::{
    CheckpointRootBasis, CheckpointStreamFooter, CheckpointWalSourceRange,
    PersistedCompactionProductRole, PhysicalCheckpointIdentity, PhysicalCheckpointSource,
};

use crate::validation::{
    IntegrityValidatedCheckpointBinding, IntegrityValidatedCheckpointBindingCompaction,
    IntegrityValidatedCheckpointDirtyBasis, IntegrityValidatedCheckpointFooter,
    IntegrityValidatedCheckpointStreamHeader, UntrustedPhysicalArtifact,
};
use crate::{
    validate_checkpoint_footer, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, PhysicalByteRange, PhysicalIntegrityRejection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedCheckpointStream {
    source: PhysicalCheckpointSource,
    footer: CheckpointStreamFooter,
    encoded_bytes: u64,
    encoded_digest: [u8; 32],
    compaction_cutover: VerifiedCheckpointCompactionCutover,
    certificate_records: Box<[Box<[u8]>]>,
}

impl VerifiedCheckpointStream {
    /// Certificate backing only. A caller sharing this stream counts it once,
    /// plus the stream and Arc header; cumulative encoded bytes are not resident.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.certificate_records.iter().try_fold(
            u64::try_from(std::mem::size_of_val(&*self.certificate_records)).ok()?,
            |bytes, frame| bytes.checked_add(u64::try_from(frame.len()).ok()?),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifiedCheckpointStreamAssemblyDenial {
    SourceIdentityMismatch,
    RecordScopeMismatch,
    InputIncarnationMismatch,
    FooterBasisMismatch(PhysicalIntegrityRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedCheckpointCompactionCutover {
    checkpoint: PhysicalCheckpointIdentity,
    root: CheckpointRootBasis,
    checkpoint_wal: CheckpointWalSourceRange,
    product_generation: u64,
    wal_cutoff_lsn_exclusive: u64,
}

impl VerifiedCheckpointStream {
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

    pub fn assemble_from_validated_records_with_certificates<'records, 'media>(
        complete_stream: UntrustedPhysicalArtifact<'media>,
        header: &'records IntegrityValidatedCheckpointStreamHeader<'media>,
        dirty: &'records [&'records IntegrityValidatedCheckpointDirtyBasis<'media>],
        compaction: &'records IntegrityValidatedCheckpointBindingCompaction<'media>,
        bindings: &'records [&'records IntegrityValidatedCheckpointBinding<'media>],
        certificates: &'records [(PhysicalByteRange, &'media [u8])],
        footer: &'records IntegrityValidatedCheckpointFooter<'media>,
    ) -> Result<Self, VerifiedCheckpointStreamAssemblyDenial> {
        let source = header.source();
        let identity = source.identity();
        if footer.footer().identity() != identity {
            return Err(VerifiedCheckpointStreamAssemblyDenial::SourceIdentityMismatch);
        }
        let mut next_offset =
            require_record(complete_stream, header.scope().byte_range(), 0, |input| {
                header.matches_input(input)
            })?;
        for record in dirty {
            next_offset = require_record(
                complete_stream,
                record.scope().byte_range(),
                next_offset,
                |input| record.matches_input(input),
            )?;
        }
        next_offset = require_record(
            complete_stream,
            compaction.scope().byte_range(),
            next_offset,
            |input| compaction.matches_input(input),
        )?;
        for record in bindings {
            let range = record.scope().byte_range();
            next_offset = require_record(complete_stream, range, next_offset, |input| {
                record.matches_input(input)
            })?;
        }
        let mut retained_certificates = Vec::with_capacity(certificates.len());
        for &(range, bytes) in certificates {
            next_offset = require_record(complete_stream, range, next_offset, |input| {
                input.bytes() == bytes
            })?;
            retained_certificates.push(bytes.to_vec().into_boxed_slice());
        }
        let footer_range = footer.scope().byte_range();
        next_offset = require_record(complete_stream, footer_range, next_offset, |input| {
            footer.matches_input(input)
        })?;
        if next_offset != complete_stream.byte_count() {
            return Err(VerifiedCheckpointStreamAssemblyDenial::RecordScopeMismatch);
        }
        let footer_input = bounded(complete_stream, footer_range)?;
        let basis = CheckpointFooterValidationBasis::from_record_references(
            header, dirty, compaction, bindings,
        )
        .with_certificates(certificates);
        if let CheckpointFooterIntegrityValidation::Rejected(rejection) =
            validate_checkpoint_footer(footer_input, footer.scope(), basis).0
        {
            return Err(VerifiedCheckpointStreamAssemblyDenial::FooterBasisMismatch(
                rejection,
            ));
        }
        let compaction_cutover = VerifiedCheckpointCompactionCutover {
            checkpoint: identity,
            root: source.root(),
            checkpoint_wal: source.wal(),
            product_generation: footer.footer().binding_compaction_generation(),
            wal_cutoff_lsn_exclusive: footer.footer().binding_wal_cutoff_lsn_exclusive(),
        };
        Ok(Self {
            source,
            footer: footer.footer(),
            encoded_bytes: complete_stream.byte_count(),
            encoded_digest: worth_store_physical_format::checkpoint_stream_encoded_digest(
                complete_stream.bytes(),
            ),
            compaction_cutover,
            certificate_records: retained_certificates.into_boxed_slice(),
        })
    }

    pub const fn source(&self) -> PhysicalCheckpointSource {
        self.source
    }

    pub const fn footer(&self) -> CheckpointStreamFooter {
        self.footer
    }

    pub const fn encoded_bytes(&self) -> u64 {
        self.encoded_bytes
    }

    pub const fn encoded_digest(&self) -> [u8; 32] {
        self.encoded_digest
    }

    pub const fn compaction_cutover(&self) -> VerifiedCheckpointCompactionCutover {
        self.compaction_cutover
    }

    pub fn certificate_records(&self) -> &[Box<[u8]>] {
        &self.certificate_records
    }
}

impl VerifiedCheckpointCompactionCutover {
    pub const fn checkpoint(self) -> PhysicalCheckpointIdentity {
        self.checkpoint
    }

    pub const fn root(self) -> CheckpointRootBasis {
        self.root
    }

    pub const fn checkpoint_wal(self) -> CheckpointWalSourceRange {
        self.checkpoint_wal
    }

    pub const fn product_role(self) -> PersistedCompactionProductRole {
        PersistedCompactionProductRole::OperationBindingIndex
    }

    pub const fn product_generation(self) -> u64 {
        self.product_generation
    }

    pub const fn wal_cutoff_lsn_exclusive(self) -> u64 {
        self.wal_cutoff_lsn_exclusive
    }
}

fn require_record(
    complete: UntrustedPhysicalArtifact<'_>,
    range: PhysicalByteRange,
    expected_offset: u64,
    matches: impl FnOnce(UntrustedPhysicalArtifact<'_>) -> bool,
) -> Result<u64, VerifiedCheckpointStreamAssemblyDenial> {
    if range.offset() != expected_offset {
        return Err(VerifiedCheckpointStreamAssemblyDenial::RecordScopeMismatch);
    }
    let input = bounded(complete, range)?;
    if !matches(input) {
        return Err(VerifiedCheckpointStreamAssemblyDenial::InputIncarnationMismatch);
    }
    Ok(range.end_exclusive())
}

fn bounded<'media>(
    complete: UntrustedPhysicalArtifact<'media>,
    range: PhysicalByteRange,
) -> Result<UntrustedPhysicalArtifact<'media>, VerifiedCheckpointStreamAssemblyDenial> {
    complete
        .bytes()
        .get(range.offset() as usize..range.end_exclusive() as usize)
        .map(UntrustedPhysicalArtifact::from_bounded_bytes)
        .ok_or(VerifiedCheckpointStreamAssemblyDenial::RecordScopeMismatch)
}
