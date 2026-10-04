//! The identity of a released generation never publishes again. The selected
//! release denies resume and abandonment of its session, while its head
//! stands and after it retired. The head retires only once the declaration is
//! durably expired, and the declaration stays selected: its object is never
//! declared again.

use std::num::NonZeroU64;

use worth_proof::AdmittedBlobReleaseProof;
use worth_store::physical_runtime::{
    BlobIngestFailure, BlobIngestSession, BlobResumeFailure, BlobResumeLimits, BlobResumeToken,
    BlobTerminalFailure, BlobTerminalHeadRetirementReceipt, BlobTerminalLimits,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1};

use super::{
    assert_settled_absence, begin, checkpoint, deadline, denied, fixed, observe, publish_resumable,
    release_batch, release_to_terminal, retire, settle, Denial, Failure, Published, CHUNK, SCOPE,
    WHOLE_GENERATION,
};
use crate::blob_frontier::selected_blob_records;
use crate::fixture::{admitted_blob_scope, placement, serving_from_initialization};

/// Longer than the checkpoints one release completes, so the released
/// session is still inside its resume horizon when its head becomes terminal.
const LONG_HORIZON: u64 = 12;

/// The retirement must deny for the unexpired declaration alone, before any
/// effect. Returns how many more checkpoints the declaration stays unexpired.
fn unexpired_checkpoints(
    serving: &ServingPhysicalRuntime,
    proof: &AdmittedBlobReleaseProof,
) -> u64 {
    let failure = denied(serving, proof);
    let Failure::Denied(Denial::IdentityDeclarationNotExpired {
        selected_checkpoint_sequence,
        maximum_checkpoint_sequence,
    }) = failure
    else {
        panic!("an unexpired declaration keeps the tombstone: {failure:?}");
    };
    assert!(selected_checkpoint_sequence <= maximum_checkpoint_sequence);
    maximum_checkpoint_sequence - selected_checkpoint_sequence
}

/// Completes the checkpoints, keyed from `first_key`, that durably expire the
/// still unexpired declaration, then retires its head.
fn retire_once_expired(
    serving: &ServingPhysicalRuntime,
    proof: &AdmittedBlobReleaseProof,
    first_key: u8,
) -> BlobTerminalHeadRetirementReceipt {
    for round in 0..=unexpired_checkpoints(serving, proof) {
        checkpoint(serving, first_key + round as u8);
    }
    retire(serving, proof).expect("the declaration is durably expired")
}

fn resume(
    serving: &ServingPhysicalRuntime,
    token: BlobResumeToken,
) -> Result<BlobIngestSession<'_>, BlobResumeFailure> {
    serving.blobs().unwrap().resume_ingest(
        token,
        &admitted_blob_scope(SCOPE),
        placement(),
        CHUNK as u64,
        deadline(),
        BlobResumeLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(1 << 20).unwrap(),
        ),
    )
}

/// Resume, abort and expiry of a released session each answer the release,
/// and none of them appends: the WAL end, the selected root and the release
/// ledger stay exactly as they were.
fn assert_release_is_the_sessions_fate(serving: &ServingPhysicalRuntime, token: BlobResumeToken) {
    let before = fixed(serving);
    let scope = admitted_blob_scope(SCOPE);
    let resumed = resume(serving, token).err();
    assert!(
        matches!(resumed, Some(BlobResumeFailure::AlreadyReleased)),
        "a released session resumed: {resumed:?}"
    );
    let blobs = serving.blobs().unwrap();
    let limits = || BlobTerminalLimits::new(NonZeroU64::new(128).unwrap());
    for abandoned in [
        blobs.abort_ingest(token, &scope, placement(), deadline(), limits()),
        blobs.expire_ingest(token, &scope, placement(), deadline(), limits()),
    ] {
        let abandoned = abandoned.err();
        assert!(
            matches!(abandoned, Some(BlobTerminalFailure::AlreadyReleased)),
            "a released session was abandoned: {abandoned:?}"
        );
    }
    assert_eq!(
        fixed(serving),
        before,
        "a denied session admission changed the WAL, the root or the release ledger"
    );
}

/// The selected publications of the identity: of its object, or by the
/// session its still selected declaration names.
fn selected_publications(serving: &ServingPhysicalRuntime, published: &Published) -> usize {
    let object = published.object.bytes();
    let selected = selected_blob_records(serving);
    let records = || {
        selected
            .iter()
            .filter_map(|(_, bytes)| decode_blob_record(bytes).ok())
    };
    let session = records()
        .find_map(|record| match record {
            BlobRecordV1::SessionDeclared(declared) if declared.object() == object => {
                Some(declared.session())
            }
            _ => None,
        })
        .expect("the declaration of the object stays selected");
    records()
        .filter(|record| {
            matches!(
                record,
                BlobRecordV1::GenerationPublished(publication)
                    if publication.object() == object || publication.session() == session
            )
        })
        .count()
}

/// After its head retired the identity stays ended: every session entry
/// answers the release, the object cannot be declared again, the release and
/// the retirement repeat without effect, and no publication of the identity
/// is selected.
fn assert_retired_identity_stays_ended(serving: &ServingPhysicalRuntime, published: &Published) {
    assert_release_is_the_sessions_fate(serving, published.token);
    let before = fixed(serving);
    let begun = begin(serving, published.object, LONG_HORIZON).err();
    assert!(
        matches!(begun, Some(BlobIngestFailure::ObjectAlreadyDeclared)),
        "the retired object was declared again: {begun:?}"
    );
    assert_eq!(
        fixed(serving),
        before,
        "a denied declaration left an effect"
    );
    assert_settled_absence(serving, &published.proof);
    assert_eq!(selected_publications(serving, published), 0);
}

#[test]
fn a_released_session_never_resumes_and_its_head_retires_once_its_declaration_expires() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let published = publish_resumable(&serving, 0x31, LONG_HORIZON);
    let (proof, token) = (&published.proof, published.token);
    release_to_terminal(&serving, proof);
    let tombstone = observe(&serving);
    assert_eq!(tombstone.effective_heads().0, 1);

    let inside = unexpired_checkpoints(&serving, proof);
    assert!(inside > 0, "the release outlived the declaration horizon");
    assert_release_is_the_sessions_fate(&serving, token);
    for round in 0..inside {
        checkpoint(&serving, 0x40 + round as u8);
    }
    // The declaration is still unexpired at its maximum itself.
    assert_eq!(unexpired_checkpoints(&serving, proof), 0);
    assert_release_is_the_sessions_fate(&serving, token);
    assert_eq!(
        observe(&serving).effective_heads(),
        tombstone.effective_heads(),
        "the head stays a tombstone inside the horizon"
    );

    checkpoint(&serving, 0x81);
    let receipt = retire(&serving, proof).expect("the declaration is durably expired");
    assert!(receipt.head_tree_emptied());
    for key in [0x82, 0x83] {
        assert_retired_identity_stays_ended(&serving, &published);
        assert_eq!(observe(&serving).effective_heads().0, 0);
        checkpoint(&serving, key);
    }
    assert_retired_identity_stays_ended(&serving, &published);
    assert_eq!(observe(&serving).checkpoint_heads().0, 0);
    serving.close();
}

#[test]
fn a_release_still_in_flight_is_already_the_fate_of_its_session() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let Published { proof, token, .. } = publish_resumable(&serving, 0x31, LONG_HORIZON);
    let mut first = release_batch(&serving, &proof, 1);
    assert!(first.remaining_payload_records() > 0);
    assert_release_is_the_sessions_fate(&serving, token);
    settle(&serving, &mut first);
    assert_release_is_the_sessions_fate(&serving, token);
    release_to_terminal(&serving, &proof);
    assert_release_is_the_sessions_fate(&serving, token);
    serving.close();
}

/// A second generation reuses the chunks of the first. The reuser is
/// unaffected by the source's release, the source stays denied while the
/// reuser holds its chunks, and each head retires on its own declaration.
#[test]
fn a_session_reusing_the_released_chunks_is_unaffected_and_both_heads_retire() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let source = publish_resumable(&serving, 0x31, LONG_HORIZON);
    checkpoint(&serving, 0x30);
    let reuser = publish_resumable(&serving, 0x31, LONG_HORIZON);
    let reuse_claims = selected_blob_records(&serving)
        .iter()
        .filter(|(_, bytes)| {
            matches!(
                decode_blob_record(bytes),
                Ok(BlobRecordV1::ChunkReuseClaimV2(_))
            )
        })
        .count();
    assert_eq!(reuse_claims, 2, "the reuser wrote chunks of its own");

    let mut first = release_batch(&serving, &source.proof, WHOLE_GENERATION);
    assert!(
        first.remaining_payload_records() > 0,
        "the reuser holds the source's chunks"
    );
    settle(&serving, &mut first);
    assert_release_is_the_sessions_fate(&serving, source.token);
    assert_eq!(selected_publications(&serving, &source), 0);
    let resumed = resume(&serving, reuser.token).err();
    assert!(
        matches!(resumed, Some(BlobResumeFailure::AlreadyPublished)),
        "the source's release reached the reuser: {resumed:?}"
    );
    assert_eq!(selected_publications(&serving, &reuser), 1);
    assert!(matches!(
        denied(&serving, &reuser.proof),
        Failure::Denied(Denial::IdentityPublicationSelected)
    ));

    release_to_terminal(&serving, &reuser.proof);
    release_to_terminal(&serving, &source.proof);
    assert_eq!(observe(&serving).effective_heads().0, 2);
    for (published, first_key) in [(&source, 0x40), (&reuser, 0x60)] {
        assert_release_is_the_sessions_fate(&serving, published.token);
        retire_once_expired(&serving, &published.proof, first_key);
    }
    for key in [0x82, 0x83] {
        for published in [&source, &reuser] {
            assert_retired_identity_stays_ended(&serving, published);
        }
        assert_eq!(observe(&serving).effective_heads().0, 0);
        checkpoint(&serving, key);
    }
    assert_eq!(observe(&serving).checkpoint_heads().0, 0);
    serving.close();
}
