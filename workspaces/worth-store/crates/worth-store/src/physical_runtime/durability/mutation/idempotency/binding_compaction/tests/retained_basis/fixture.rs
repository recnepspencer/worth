//! Exact C9 record assembly for issued-key codec histories, not WAL authority.

use worth_store_physical_format::*;
use worth_store_physical_integrity::*;

pub(super) fn with_bindings<R>(
    source: PhysicalCheckpointSource,
    payloads: &[&[u8]],
    run: impl FnOnce(
        ValidatedCheckpointStreamAssembly<'_, '_>,
        &[IntegrityValidatedCheckpointBinding<'_>],
        &[UntrustedPhysicalArtifact<'_>],
    ) -> R,
) -> R {
    let identity = source.identity();
    let (encoder, header_bytes) = CheckpointStreamEncoder::begin(source);
    let (mut encoder, compaction_bytes) =
        encoder.begin_binding_compaction(CheckpointBindingCompactionHeader::new(1, 2).unwrap());
    let binding_bytes: Vec<_> = payloads
        .iter()
        .map(|payload| encoder.encode_binding_record(payload).unwrap())
        .collect();
    let (_, footer_bytes) = encoder.finish();
    let compaction_at = header_bytes.len();
    let binding_at = compaction_at + compaction_bytes.len();
    let mut bytes = header_bytes;
    bytes.extend_from_slice(&compaction_bytes);
    let mut ranges = Vec::new();
    for binding in binding_bytes {
        let start = bytes.len();
        bytes.extend_from_slice(&binding);
        ranges.push((start, bytes.len()));
    }
    let footer_at = bytes.len();
    bytes.extend_from_slice(&footer_bytes);
    let input = |start, end| UntrustedPhysicalArtifact::from_bounded_bytes(&bytes[start..end]);
    let range = |start: usize, end: usize| {
        PhysicalByteRange::new(start as u64, (end - start) as u64).unwrap()
    };
    let CheckpointStreamHeaderIntegrityValidation::Intact(header) =
        validate_checkpoint_stream_header(
            input(0, compaction_at),
            PhysicalArtifactScope::checkpoint_stream_header(
                CheckpointStreamHeaderScopeIdentity::known(identity),
                range(0, compaction_at),
            ),
        )
        .0
    else {
        panic!("canonical C9 header must admit");
    };
    let CheckpointBindingCompactionIntegrityValidation::Intact(compaction) =
        validate_checkpoint_binding_compaction(
            input(compaction_at, binding_at),
            PhysicalArtifactScope::checkpoint_binding_compaction(
                identity,
                range(compaction_at, binding_at),
            ),
        )
        .0
    else {
        panic!("canonical C9 compaction must admit");
    };
    let bindings: Vec<_> = ranges
        .iter()
        .map(|&(start, end)| {
            let CheckpointBindingIntegrityValidation::Intact(binding) =
                validate_checkpoint_binding(
                    input(start, end),
                    PhysicalArtifactScope::checkpoint_binding(identity, range(start, end)),
                )
                .0
            else {
                panic!("issued-key history must have valid C9 framing");
            };
            binding
        })
        .collect();
    let references: Vec<_> = bindings.iter().collect();
    let CheckpointFooterIntegrityValidation::Intact(footer) = validate_checkpoint_footer(
        input(footer_at, bytes.len()),
        PhysicalArtifactScope::checkpoint_footer(identity, range(footer_at, bytes.len())),
        CheckpointFooterValidationBasis::from_record_references(
            &header,
            &[],
            &compaction,
            &references,
        ),
    )
    .0
    else {
        panic!("exact C9 aggregate must admit");
    };
    let assembly = VerifiedCheckpointStream::validate_records_with_certificates(
        input(0, bytes.len()),
        &header,
        &[],
        &compaction,
        &references,
        &[],
        &footer,
    )
    .unwrap();
    let exact: Vec<_> = ranges
        .iter()
        .map(|&(start, end)| input(start, end))
        .collect();
    run(assembly, &bindings, &exact)
}
