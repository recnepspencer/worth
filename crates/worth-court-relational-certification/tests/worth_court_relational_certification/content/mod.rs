//! Generic ordered exact samples exercise Native's actual frame and request owners.
mod admission;
mod backing_refusal;
mod carried_export;
mod checkpoint_capture_controls;
mod checkpoint_controls;
mod checkpoint_corruption;
mod cold_envelope;
mod durable_publication;
pub(crate) mod expected;
pub(crate) mod fixture;
pub(crate) mod historical_kind;
mod identity_conflict;
mod journal_recovery;
mod lifecycle;
mod recovery;
mod retained;
mod schema_install;
mod selected_value;
pub(crate) mod transaction;
mod unreferenced_attachment;
use fixture::*;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{CanonicalDigestId, ContentFormat, ContentRefId};
use worth_relational::facade::content::*;

pub(crate) fn original(cancel: &CancellationSource) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(64 * 1024 * 1024)),
        cancel.token(),
        None,
    )
}

#[test]
fn content_literal_frame_and_digest_are_independent_of_the_writer() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let format = format("sample-grid", 1, false);
    let frame = RelationalContentFrame::encode(&execution, &format, &grid()).unwrap();
    let expected = expected::FRAME;
    assert_eq!(frame.bytes(), expected);
    let mut oracle = Sha256::new();
    oracle.update(b"worth.native.content.v1\0");
    oracle.update(expected);
    let expected_id = ContentRefId(oracle.finalize().into());
    assert_eq!(frame.identity(), expected_id);
    // Golden independently computed with Python hashlib/OpenSSL over the literal frame.
    assert_eq!(
        frame.identity().to_string(),
        "f4fb54508d898711c5770a58e4cff9989c1c302e7969d842891c0f5f02dc78e8"
    );
    let read = RelationalContentFrame::readmit(&execution, &format, expected_id, expected).unwrap();
    assert_eq!(read.payload(), &expected[35..]);
    assert!(frame.same_content(&execution, &read).unwrap());
}

#[test]
fn content_corruption_format_version_and_missing_bytes_refuse() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let good = format("sample-grid", 1, false);
    let frame = RelationalContentFrame::encode(&execution, &good, &grid()).unwrap();
    let mut corrupt = frame.bytes().to_vec();
    *corrupt.last_mut().unwrap() ^= 1;
    assert_eq!(
        RelationalContentFrame::readmit(&execution, &good, frame.identity(), &corrupt).unwrap_err(),
        RelationalContentDenial::IntegrityMismatch
    );
    let wrong = format("other-grid", 1, false);
    assert_eq!(
        RelationalContentFrame::readmit(&execution, &wrong, frame.identity(), frame.bytes())
            .unwrap_err(),
        RelationalContentDenial::FormatMismatch
    );
    let version = format("sample-grid", 2, false);
    assert_eq!(
        RelationalContentFrame::readmit(&execution, &version, frame.identity(), frame.bytes())
            .unwrap_err(),
        RelationalContentDenial::UnsupportedVersion
    );
    assert_eq!(
        RelationalContentFrame::readmit(&execution, &good, frame.identity(), &[]).unwrap_err(),
        RelationalContentDenial::NonCanonicalContent
    );
}

#[test]
fn content_altered_canonical_result_is_caught_by_correspondence_verifier() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    assert_eq!(
        RelationalContentFrame::encode(&execution, &format("sample-grid", 1, true), &grid())
            .unwrap_err(),
        RelationalContentDenial::EncodingInconsistency
    );
    assert_eq!(
        original.memory().unwrap().available_memory_bytes(),
        Some(64 * 1024 * 1024)
    );
}

#[test]
fn content_original_cancellation_releases_failed_custody_and_aliases_retain_success() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let format = format("sample-grid", 1, false);
    let frame = RelationalContentFrame::encode(&execution, &format, &grid()).unwrap();
    let held = original.memory().unwrap().available_memory_bytes().unwrap();
    assert!(held < 64 * 1024 * 1024);
    let alias = frame.clone();
    drop(frame);
    assert_eq!(
        original.memory().unwrap().available_memory_bytes(),
        Some(held)
    );
    cancel.cancel();
    assert!(matches!(
        RelationalContentFrame::encode(&execution, &format, &grid()),
        Err(RelationalContentDenial::Execution(_))
    ));
    drop(alias);
    assert_eq!(
        original.memory().unwrap().available_memory_bytes(),
        Some(64 * 1024 * 1024)
    );
}

#[test]
fn content_format_admission_is_unique_and_two_field_bindings_share_it() {
    let mut formats = RelationalContentFormatRegistry::default();
    let codec = Arc::new(Codec {
        descriptor: ContentFormat::new("sample-grid", 1).unwrap(),
        alter: false,
    });
    let id = CanonicalDigestId::new([1; 32]);
    formats.register(id, codec.clone()).unwrap();
    formats
        .bind_field(CanonicalDigestId::new([2; 32]), id)
        .unwrap();
    formats
        .bind_field(CanonicalDigestId::new([3; 32]), id)
        .unwrap();
    assert_eq!(
        formats
            .register(CanonicalDigestId::new([4; 32]), codec)
            .unwrap_err(),
        RelationalContentDenial::DuplicateContentFormat
    );
}

#[test]
fn content_identifiers_have_one_ascii_spelling() {
    for name in ["", "Grid", "grid/\u{e9}", "grid space"] {
        assert!(ContentFormat::new(name, 1).is_err());
    }
    assert!(ContentFormat::new("grid", 0).is_err());
    assert!(ContentFormat::new("grid/v1.points-0", 1).is_ok());
    assert_eq!(
        ContentRefId::from_hex(&"a1".repeat(32)),
        Some(ContentRefId([0xa1; 32]))
    );
    assert!(ContentRefId::from_hex(&"A1".repeat(32)).is_none());
    assert!(ContentRefId::from_hex("42").is_none());
}

mod schema_transition;

mod historical_custody;

mod preimage_work;

mod recovered_reference;
