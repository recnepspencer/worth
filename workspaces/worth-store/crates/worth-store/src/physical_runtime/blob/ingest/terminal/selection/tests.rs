//! A selected release of the generation a session published denies abort and
//! expiry, whether its manifest fits the control scan or was deferred.

use super::super::super::selected_session::fixture::{basis, record, released_manifest};
use super::*;

fn scanned(session: [u8; 16], object: [u8; 16]) -> Result<(), BlobTerminalFailure> {
    let (token, declaration) = basis();
    let frame = released_manifest(token.store, session, object, 2).encode();
    assert!(frame.len() <= CONTROL_SCAN_BYTES);
    let (mut found_declaration, mut existing) = (false, None);
    let fate = inspect_control(
        &frame,
        record(7, 1),
        token,
        declaration,
        None,
        &mut found_declaration,
        &mut existing,
    );
    assert!(!found_declaration && existing.is_none());
    fate
}

fn deferred(session: [u8; 16], object: [u8; 16]) -> Result<(), BlobTerminalFailure> {
    let (token, declaration) = basis();
    let frame = released_manifest(token.store, session, object, 64).encode();
    assert!(frame.len() > CONTROL_SCAN_BYTES);
    deferred_fate(&frame[..MANIFEST_V3_WINDOW], &token, declaration)
}

#[test]
fn a_selected_release_naming_the_session_or_its_object_denies_abandonment() {
    let (token, declaration) = basis();
    for (session, object) in [
        (token.session, declaration.object()),
        (token.session, [0x52; 16]),
        ([0x51; 16], declaration.object()),
    ] {
        assert!(matches!(
            scanned(session, object),
            Err(BlobTerminalFailure::AlreadyReleased)
        ));
        assert!(matches!(
            deferred(session, object),
            Err(BlobTerminalFailure::AlreadyReleased)
        ));
    }
}

#[test]
fn a_selected_release_of_another_generation_is_not_this_sessions_fate() {
    assert!(scanned([0x51; 16], [0x52; 16]).is_ok());
    assert!(deferred([0x51; 16], [0x52; 16]).is_ok());
}

#[test]
fn a_deferred_manifest_whose_source_window_is_damaged_is_a_format_denial() {
    let (token, declaration) = basis();
    let frame = released_manifest(token.store, token.session, declaration.object(), 64).encode();
    let mut window = frame[..MANIFEST_V3_WINDOW].to_vec();
    window[MANIFEST_V3_WINDOW - 100] ^= 1;
    assert!(matches!(
        deferred_fate(&window, &token, declaration),
        Err(BlobTerminalFailure::Format(_))
    ));
    assert!(matches!(
        deferred_fate(&frame[..MANIFEST_V3_WINDOW - 1], &token, declaration),
        Err(BlobTerminalFailure::Format(
            BlobRecordDenial::LengthMismatch
        ))
    ));
}

#[test]
fn only_kinds_that_cannot_decide_the_fate_stay_unread_when_deferred() {
    let (token, declaration) = basis();
    let prefix = |kind: u8| {
        let mut prefix = [0_u8; KIND_PREFIX_BYTES];
        prefix[..8].copy_from_slice(BLOB_MAGIC);
        prefix[8] = kind;
        prefix
    };
    for kind in [
        BlobRecordKind::Chunk,
        BlobRecordKind::TreeNode,
        BlobRecordKind::DropSetManifest,
        BlobRecordKind::ReclaimDescriptor,
        BlobRecordKind::DropSetManifestV2,
        BlobRecordKind::OriginalDropReserved,
        BlobRecordKind::DedupeQuarantine,
        BlobRecordKind::ReclaimDescriptorV2,
        BlobRecordKind::ReclaimDescriptorV3,
    ] {
        assert!(
            deferred_fate(&prefix(kind as u8), &token, declaration).is_ok(),
            "{kind:?} cannot decide the fate"
        );
    }
    let deciding = [
        BlobRecordKind::SessionDeclared,
        BlobRecordKind::GenerationPublished,
        BlobRecordKind::SessionFrontier,
        BlobRecordKind::SessionAbandoned,
        BlobRecordKind::ChunkReuseClaim,
        BlobRecordKind::ChunkReuseClaimV2,
    ]
    .map(|kind| kind as u8);
    // A byte no kind names is refused with the kinds that decide the fate.
    for kind in deciding.into_iter().chain([0, u8::MAX]) {
        assert!(
            matches!(
                deferred_fate(&prefix(kind), &token, declaration),
                Err(BlobTerminalFailure::Format(BlobRecordDenial::FrameTooLarge))
            ),
            "kind byte {kind} stayed unread"
        );
    }
    assert!(deferred_fate(&[0x55; KIND_PREFIX_BYTES], &token, declaration).is_ok());
}
