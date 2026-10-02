//! Allocation-free complete-stream admission and exact prepared-certificate retention.

use super::{
    IntegrityValidatedCheckpointBinding, IntegrityValidatedCheckpointBindingCompaction,
    IntegrityValidatedCheckpointDirtyBasis, IntegrityValidatedCheckpointFooter,
    IntegrityValidatedCheckpointStreamHeader, PhysicalByteRange, UntrustedPhysicalArtifact,
    VerifiedCheckpointCompactionCutover, VerifiedCheckpointFacts, VerifiedCheckpointStream,
    VerifiedCheckpointStreamAssemblyDenial as Denial,
};
use crate::{
    validate_checkpoint_footer, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis,
};

#[derive(Debug)]
pub struct ValidatedCheckpointStreamAssembly<'records, 'media> {
    facts: VerifiedCheckpointFacts,
    certificates: &'records [(PhysicalByteRange, &'media [u8])],
}

impl<'records, 'media> ValidatedCheckpointStreamAssembly<'records, 'media> {
    pub const fn facts(&self) -> VerifiedCheckpointFacts {
        self.facts
    }

    pub const fn certificates(&self) -> &'records [(PhysicalByteRange, &'media [u8])] {
        self.certificates
    }

    pub fn retain_prepared_certificates(
        self,
        certificates: Box<[Box<[u8]>]>,
    ) -> Result<VerifiedCheckpointStream, Denial> {
        if certificates.len() != self.certificates.len() {
            return Err(Denial::RecordScopeMismatch);
        }
        if !certificates
            .iter()
            .zip(self.certificates)
            .all(|(prepared, (_, observed))| prepared.as_ref() == *observed)
        {
            return Err(Denial::InputIncarnationMismatch);
        }
        Ok(VerifiedCheckpointStream {
            facts: self.facts,
            certificate_records: certificates,
        })
    }
}

pub(super) fn validate<'records, 'media>(
    complete_stream: UntrustedPhysicalArtifact<'media>,
    header: &'records IntegrityValidatedCheckpointStreamHeader<'media>,
    dirty: &'records [&'records IntegrityValidatedCheckpointDirtyBasis<'media>],
    compaction: &'records IntegrityValidatedCheckpointBindingCompaction<'media>,
    bindings: &'records [&'records IntegrityValidatedCheckpointBinding<'media>],
    certificates: &'records [(PhysicalByteRange, &'media [u8])],
    footer: &'records IntegrityValidatedCheckpointFooter<'media>,
) -> Result<ValidatedCheckpointStreamAssembly<'records, 'media>, Denial> {
    let source = header.source();
    let identity = source.identity();
    if footer.footer().identity() != identity {
        return Err(Denial::SourceIdentityMismatch);
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
        next_offset = require_record(
            complete_stream,
            record.scope().byte_range(),
            next_offset,
            |input| record.matches_input(input),
        )?;
    }
    for &(range, bytes) in certificates {
        next_offset = require_record(complete_stream, range, next_offset, |input| {
            input.bytes() == bytes
        })?;
    }
    let footer_range = footer.scope().byte_range();
    next_offset = require_record(complete_stream, footer_range, next_offset, |input| {
        footer.matches_input(input)
    })?;
    if next_offset != complete_stream.byte_count() {
        return Err(Denial::RecordScopeMismatch);
    }
    let basis = CheckpointFooterValidationBasis::from_record_references(
        header, dirty, compaction, bindings,
    )
    .with_certificates(certificates);
    if let CheckpointFooterIntegrityValidation::Rejected(rejection) = validate_checkpoint_footer(
        bounded(complete_stream, footer_range)?,
        footer.scope(),
        basis,
    )
    .0
    {
        return Err(Denial::FooterBasisMismatch(rejection));
    }
    Ok(ValidatedCheckpointStreamAssembly {
        facts: VerifiedCheckpointFacts {
            source,
            footer: footer.footer(),
            encoded_bytes: complete_stream.byte_count(),
            encoded_digest: worth_store_physical_format::checkpoint_stream_encoded_digest(
                complete_stream.bytes(),
            ),
            compaction_cutover: VerifiedCheckpointCompactionCutover {
                checkpoint: identity,
                root: source.root(),
                checkpoint_wal: source.wal(),
                product_generation: footer.footer().binding_compaction_generation(),
                wal_cutoff_lsn_exclusive: footer.footer().binding_wal_cutoff_lsn_exclusive(),
            },
        },
        certificates,
    })
}

fn require_record(
    complete: UntrustedPhysicalArtifact<'_>,
    range: PhysicalByteRange,
    expected_offset: u64,
    matches: impl FnOnce(UntrustedPhysicalArtifact<'_>) -> bool,
) -> Result<u64, Denial> {
    if range.offset() != expected_offset {
        return Err(Denial::RecordScopeMismatch);
    }
    if !matches(bounded(complete, range)?) {
        return Err(Denial::InputIncarnationMismatch);
    }
    Ok(range.end_exclusive())
}

fn bounded<'media>(
    complete: UntrustedPhysicalArtifact<'media>,
    range: PhysicalByteRange,
) -> Result<UntrustedPhysicalArtifact<'media>, Denial> {
    complete
        .bytes()
        .get(range.offset() as usize..range.end_exclusive() as usize)
        .map(UntrustedPhysicalArtifact::from_bounded_bytes)
        .ok_or(Denial::RecordScopeMismatch)
}
