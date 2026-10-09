use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobAbandonmentReasonV1, BlobReclaimSourceBasisV1, BlobRecordDenial, BlobSessionAbandonedV1,
    BlobSessionDeclarationV1, BLOB_RECORD_HEADER_BYTES,
};

use crate::physical_runtime::{
    durability::CompletedDurableCheckpointWitness, AdmittedBlobScope, PhysicalRecordId,
    PhysicalRecordReader, RecordByteLimit, RecordReadError, RecordReadLimits, RecordStreamFailure,
};

use super::resume::BlobResumeToken;

// The version-one declaration has 108 payload bytes. A token never grants a
// read beyond this exact selected control frame.
pub(in crate::physical_runtime::blob) const DECLARATION_FRAME_BYTES: usize =
    BLOB_RECORD_HEADER_BYTES + 108;

#[derive(Debug)]
pub(in crate::physical_runtime::blob) enum SelectedSessionFailure {
    Format(BlobRecordDenial),
    Read(RecordReadError),
    Stream(RecordStreamFailure),
    ForeignStore,
    DeclarationMismatch,
    ScopeMismatch,
}

/// Authenticates the token against the exact C.5-selected declaration and the
/// admitted caller scope. Resume's memory admission remains its own decision.
pub(in crate::physical_runtime::blob) fn read_authenticated_declaration(
    reader: &PhysicalRecordReader,
    token: &BlobResumeToken,
    scope: &AdmittedBlobScope,
) -> Result<BlobSessionDeclarationV1, SelectedSessionFailure> {
    let limit = RecordByteLimit::new(DECLARATION_FRAME_BYTES as u32)
        .expect("fixed declaration frame has positive length");
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(token.declaration_record),
            RecordReadLimits::new(limit),
        )
        .map_err(SelectedSessionFailure::Read)?;
    let mut bytes = [0_u8; DECLARATION_FRAME_BYTES];
    let mut used = 0;
    while used < bytes.len() {
        let count = stream
            .read_next(&mut bytes[used..])
            .map_err(SelectedSessionFailure::Stream)?;
        if count == 0 {
            return Err(SelectedSessionFailure::Format(BlobRecordDenial::Truncated));
        }
        used += count;
    }
    let mut excess = [0_u8; 1];
    if stream
        .read_next(&mut excess)
        .map_err(SelectedSessionFailure::Stream)?
        != 0
    {
        return Err(SelectedSessionFailure::Format(
            BlobRecordDenial::LengthMismatch,
        ));
    }
    let declaration =
        BlobSessionDeclarationV1::decode(&bytes).map_err(SelectedSessionFailure::Format)?;
    let store = reader.store_identity().bytes();
    if token.store != store || declaration.store() != store {
        return Err(SelectedSessionFailure::ForeignStore);
    }
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    if declaration.session() != token.session
        || digest != token.declaration_digest
        || declaration.chunk_size() != token.chunk_size
        || declaration.declared_bytes() != token.total_bytes
        || declaration.max_checkpoint_sequence() != token.max_checkpoint_sequence
    {
        return Err(SelectedSessionFailure::DeclarationMismatch);
    }
    if declaration.key_scope() != scope.fingerprint() {
        return Err(SelectedSessionFailure::ScopeMismatch);
    }
    Ok(declaration)
}

/// An inner terminal frame is selected fate only when its declaration binding
/// and (for expiry) completed-checkpoint coordinates are independently valid.
pub(in crate::physical_runtime::blob) fn selected_abandonment_matches(
    abandoned: BlobSessionAbandonedV1,
    token: &BlobResumeToken,
    declaration: BlobSessionDeclarationV1,
    completed: Option<CompletedDurableCheckpointWitness>,
) -> bool {
    if abandoned.store() != token.store
        || abandoned.session() != token.session
        || abandoned.declaration_record() != token.declaration_record
        || abandoned.declaration_digest() != token.declaration_digest
    {
        return false;
    }
    match abandoned.reason() {
        BlobAbandonmentReasonV1::ExplicitAbort => true,
        BlobAbandonmentReasonV1::CheckpointExpired {
            checkpoint_sequence,
        } => completed.is_some_and(|selected| {
            expiry_order_admitted(
                checkpoint_sequence.get(),
                declaration.max_checkpoint_sequence(),
                selected.sequence().get(),
            )
        }),
    }
}

/// Whether the source of a selected V3 drop-set manifest names this session
/// or its object. Such a manifest means Store released the generation this
/// session published: the session can neither resume nor be abandoned, or its
/// identity would publish a second time.
///
/// The manifest is the one release control that names the object and the
/// session; the descriptor and the reservation carry only a digest of the
/// source. A release head binds its manifest record, and the retention law
/// keeps the current head's control closure selected for as long as the head
/// exists, so this answer cannot lapse while a head of the identity stands.
pub(in crate::physical_runtime::blob) fn release_source_names_session(
    source: BlobReclaimSourceBasisV1,
    token: &BlobResumeToken,
    declaration: BlobSessionDeclarationV1,
) -> bool {
    matches!(
        source,
        BlobReclaimSourceBasisV1::ReleasedGeneration(released)
            if released.session() == token.session || released.object() == declaration.object()
    )
}

/// Pure ordering only; the caller must supply the owner-minted completed
/// checkpoint witness before this predicate may affect selected fate.
fn expiry_order_admitted(recorded: u64, maximum: u64, selected: u64) -> bool {
    recorded > maximum && recorded <= selected
}

#[cfg(test)]
#[path = "selected_session/fixture.rs"]
pub(in crate::physical_runtime::blob::ingest) mod fixture;

#[cfg(test)]
mod expiry_tests {
    use super::expiry_order_admitted;

    #[test]
    fn expiry_needs_strict_declaration_crossing_and_no_future_witness() {
        assert!(!expiry_order_admitted(9, 9, 10));
        assert!(!expiry_order_admitted(8, 9, 10));
        assert!(!expiry_order_admitted(11, 9, 10));
        assert!(expiry_order_admitted(10, 9, 10));
        assert!(expiry_order_admitted(10, 9, 12));
    }
}
