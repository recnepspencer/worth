//! The identity of a released generation never publishes again. The selected
//! release denies resume and abandonment of its session, while its head
//! stands and after it retired. The head retires only once the declaration is
//! durably expired, and the declaration stays selected: its object is never
//! declared again.

pub(crate) use std::num::NonZeroU64;

pub(crate) use worth_proof::AdmittedBlobReleaseProof;
pub(crate) use {
    worth_store::physical_runtime::BlobIngestFailure,
    worth_store::physical_runtime::BlobIngestSession,
    worth_store::physical_runtime::BlobResumeFailure,
    worth_store::physical_runtime::BlobResumeLimits,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::BlobTerminalFailure,
    worth_store::physical_runtime::BlobTerminalHeadRetirementReceipt,
    worth_store::physical_runtime::BlobTerminalLimits,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
};

pub(crate) use worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records;
pub(crate) use {
    crate::physical_blob_journeys::terminal_head_retirement::assert_settled_absence,
    crate::physical_blob_journeys::terminal_head_retirement::checkpoint,
    crate::physical_blob_journeys::terminal_head_retirement::deadline,
    crate::physical_blob_journeys::terminal_head_retirement::denied,
    crate::physical_blob_journeys::terminal_head_retirement::fixed,
    crate::physical_blob_journeys::terminal_head_retirement::observe,
    crate::physical_blob_journeys::terminal_head_retirement::publication::begin,
    crate::physical_blob_journeys::terminal_head_retirement::publication::publish_resumable,
    crate::physical_blob_journeys::terminal_head_retirement::publication::Published,
    crate::physical_blob_journeys::terminal_head_retirement::release_batch,
    crate::physical_blob_journeys::terminal_head_retirement::release_to_terminal,
    crate::physical_blob_journeys::terminal_head_retirement::retire,
    crate::physical_blob_journeys::terminal_head_retirement::settle,
    crate::physical_blob_journeys::terminal_head_retirement::CHUNK,
    crate::physical_blob_journeys::terminal_head_retirement::SCOPE,
    crate::physical_blob_journeys::terminal_head_retirement::WHOLE_GENERATION,
    worth_store::physical_runtime::BlobTerminalHeadRetirementDenial as Denial,
    worth_store::physical_runtime::BlobTerminalHeadRetirementFailure as Failure,
};
pub(crate) use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
};

/// Longer than the checkpoints one release completes, so the released
/// session is still inside its resume horizon when its head becomes terminal.
pub(crate) const LONG_HORIZON: u64 = 12;

/// The retirement must deny for the unexpired declaration alone, before any
/// effect. Returns how many more checkpoints the declaration stays unexpired.
pub(crate) fn unexpired_checkpoints(
    serving: &ServingPhysicalRuntime,
    proof: &AdmittedBlobReleaseProof,
) -> u64 {
    let failure = denied(serving, proof);
    let worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::IdentityDeclarationNotExpired {
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
pub(crate) fn retire_once_expired(
    serving: &ServingPhysicalRuntime,
    proof: &AdmittedBlobReleaseProof,
    first_key: u8,
) -> BlobTerminalHeadRetirementReceipt {
    for round in 0..=unexpired_checkpoints(serving, proof) {
        checkpoint(serving, first_key + round as u8);
    }
    retire(serving, proof).expect("the declaration is durably expired")
}

pub(crate) fn resume(
    serving: &ServingPhysicalRuntime,
    token: BlobResumeToken,
) -> Result<BlobIngestSession<'_>, BlobResumeFailure> {
    serving.blobs().unwrap().resume_ingest(
        token,
        &admitted_blob_scope(SCOPE),
        placement(),
        CHUNK as u64,
        deadline(),
        worth_store::physical_runtime::BlobResumeLimits::new(
            std::num::NonZeroU64::new(128).unwrap(),
            std::num::NonZeroU64::new(1 << 20).unwrap(),
        ),
    )
}

/// Resume, abort and expiry of a released session each answer the release,
/// and none of them appends: the WAL end, the selected root and the release
/// ledger stay exactly as they were.
pub(crate) fn assert_release_is_the_sessions_fate(
    serving: &ServingPhysicalRuntime,
    token: BlobResumeToken,
) {
    let before = fixed(serving);
    let scope = admitted_blob_scope(SCOPE);
    let resumed = resume(serving, token).err();
    assert!(
        matches!(
            resumed,
            Some(worth_store::physical_runtime::BlobResumeFailure::AlreadyReleased)
        ),
        "a released session resumed: {resumed:?}"
    );
    let blobs = serving.blobs().unwrap();
    let limits = || {
        worth_store::physical_runtime::BlobTerminalLimits::new(
            std::num::NonZeroU64::new(128).unwrap(),
        )
    };
    for abandoned in [
        blobs.abort_ingest(token, &scope, placement(), deadline(), limits()),
        blobs.expire_ingest(token, &scope, placement(), deadline(), limits()),
    ] {
        let abandoned = abandoned.err();
        assert!(
            matches!(
                abandoned,
                Some(worth_store::physical_runtime::BlobTerminalFailure::AlreadyReleased)
            ),
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
pub(crate) fn selected_publications(
    serving: &ServingPhysicalRuntime,
    published: &Published,
) -> usize {
    let object = published.object.bytes();
    let selected = selected_blob_records(serving);
    let records = || {
        selected
            .iter()
            .filter_map(|(_, bytes)| decode_blob_record(bytes).ok())
    };
    let session = records()
        .find_map(|record| match record {
            worth_store_physical_format::BlobRecordV1::SessionDeclared(declared)
                if declared.object() == object =>
            {
                Some(declared.session())
            }
            _ => None,
        })
        .expect("the declaration of the object stays selected");
    records()
        .filter(|record| {
            matches!(
                record,
                worth_store_physical_format::BlobRecordV1::GenerationPublished(publication)
                    if publication.object() == object || publication.session() == session
            )
        })
        .count()
}

/// After its head retired the identity stays ended: every session entry
/// answers the release, the object cannot be declared again, the release and
/// the retirement repeat without effect, and no publication of the identity
/// is selected.
pub(crate) fn assert_retired_identity_stays_ended(
    serving: &ServingPhysicalRuntime,
    published: &Published,
) {
    assert_release_is_the_sessions_fate(serving, published.token);
    let before = fixed(serving);
    let begun = begin(serving, published.object, 2, LONG_HORIZON).err();
    assert!(
        matches!(
            begun,
            Some(worth_store::physical_runtime::BlobIngestFailure::ObjectAlreadyDeclared)
        ),
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
