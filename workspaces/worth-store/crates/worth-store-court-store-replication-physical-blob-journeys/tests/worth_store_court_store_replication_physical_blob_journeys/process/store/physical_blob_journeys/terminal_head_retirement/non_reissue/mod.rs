//! The identity of a released generation never publishes again. The selected
//! release denies resume and abandonment of its session, while its head
//! stands and after it retired. The head retires only once the declaration is
//! durably expired, and the declaration stays selected: its object is never
//! declared again.

#[allow(unused_imports)]
use crate::physical_blob_journeys::terminal_head_retirement::non_reissue::*;

use worth_store_physical_format::decode_blob_record;

use worth_store_testkit_store::fixture::serving_from_initialization;
use worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records;
use {
    crate::physical_blob_journeys::terminal_head_retirement::checkpoint,
    crate::physical_blob_journeys::terminal_head_retirement::denied,
    crate::physical_blob_journeys::terminal_head_retirement::observe,
    crate::physical_blob_journeys::terminal_head_retirement::publication::publish_resumable,
    crate::physical_blob_journeys::terminal_head_retirement::publication::Published,
    crate::physical_blob_journeys::terminal_head_retirement::release_batch,
    crate::physical_blob_journeys::terminal_head_retirement::release_to_terminal,
    crate::physical_blob_journeys::terminal_head_retirement::retire,
    crate::physical_blob_journeys::terminal_head_retirement::settle,
    crate::physical_blob_journeys::terminal_head_retirement::WHOLE_GENERATION,
};

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
                Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaimV2(_))
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
        matches!(
            resumed,
            Some(worth_store::physical_runtime::BlobResumeFailure::AlreadyPublished)
        ),
        "the source's release reached the reuser: {resumed:?}"
    );
    assert_eq!(selected_publications(&serving, &reuser), 1);
    assert!(matches!(
        denied(&serving, &reuser.proof),
        worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::IdentityPublicationSelected)
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
