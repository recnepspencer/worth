use super::*;
use crate::facade::durability::{
    RelationalCanonicalCommitArtifactFrameRefusal as FrameRefusal,
    RelationalCanonicalCommitArtifactRefusal as Refusal,
};
use crate::facade::history::CommitId;
use crate::facade::publication::PatchStreamPosition;
use sha2::{Digest as _, Sha256};

// The frame layout is the artifact's published contract, so these offsets
// are stated independently of the encoder's own constants.
const MAGIC: std::ops::Range<usize> = 0..8;
const VERSION: std::ops::Range<usize> = 8..10;
const POSITION: std::ops::Range<usize> = 10..18;
const COMMIT: std::ops::Range<usize> = 18..26;
const DIGEST: std::ops::Range<usize> = 26..58;
const LENGTH: std::ops::Range<usize> = 58..66;
const PAYLOAD_START: usize = 66;

struct ExportedFrame {
    bytes: Vec<u8>,
    position: PatchStreamPosition,
    commit_id: CommitId,
}

/// The newest artifact of a memory-only runtime after two real commits.
fn exported_frame(execution: &ExecutionRequest<'_, '_>) -> ExportedFrame {
    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "refusal-first");
    create_entity(execution, &runtime, "refusal-second");
    let artifact = exported_artifacts(execution, &runtime)
        .pop()
        .expect("committed history exports an artifact");
    Artifact::readmit(artifact.as_bytes(), execution).expect("the untouched frame readmits");
    ExportedFrame {
        position: artifact.position(),
        commit_id: artifact.commit_id(),
        bytes: artifact.as_bytes().to_vec(),
    }
}

fn write_u64(bytes: &mut [u8], field: std::ops::Range<usize>, value: u64) {
    bytes[field].copy_from_slice(&value.to_le_bytes());
}

/// Rewrite the declared length and digest so only the intended defect
/// remains for readmission to find.
fn reseal(bytes: &mut [u8]) {
    let payload_len = (bytes.len() - PAYLOAD_START) as u64;
    write_u64(bytes, LENGTH, payload_len);
    let mut leaves = [[0; 32]; 3];
    crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload(
        &bytes[PAYLOAD_START..],
    )
    .frame_fields(|name, raw| {
        let index = match name {
            "position" => 0,
            "canonical" => 1,
            "original_inputs" => 2,
            _ => panic!("fixture emits only the three v8 commitment fields"),
        };
        leaves[index] = <[u8; 32]>::from(Sha256::digest(raw.0));
        Ok(())
    })
    .unwrap();
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.canonical.v8\0");
    digest.update(&bytes[POSITION]);
    digest.update(&bytes[COMMIT]);
    for leaf in leaves {
        digest.update(leaf);
    }
    bytes[DIGEST].copy_from_slice(&digest.finalize());
}

fn rewrite_payload(frame: &[u8], fields: &[&str], nil_canonical: bool) -> Vec<u8> {
    let mut original = [None; 3];
    crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload(
        &frame[PAYLOAD_START..],
    )
    .frame_fields(|name, raw| {
        let index = match name {
            "position" => 0,
            "canonical" => 1,
            "original_inputs" => 2,
            _ => panic!("fixture requires the v8 source map"),
        };
        original[index] = Some(raw.0);
        Ok(())
    })
    .unwrap();
    let mut output = frame[..PAYLOAD_START].to_vec();
    output.push(0x83);
    for name in fields {
        let index = match *name {
            "position" => 0,
            "canonical" => 1,
            "original_inputs" => 2,
            _ => panic!("fixture only rewrites known fields"),
        };
        output.push(0xa0 | name.len() as u8);
        output.extend_from_slice(name.as_bytes());
        output.extend_from_slice(if index == 1 && nil_canonical {
            &[0xc0]
        } else {
            original[index].unwrap()
        });
    }
    reseal(&mut output);
    output
}

fn refusal(bytes: &[u8], execution: &ExecutionRequest<'_, '_>) -> Refusal {
    Artifact::readmit(bytes, execution).expect_err("defective artifact must refuse")
}

#[test]
fn frame_defects_refuse_before_decoding_each_with_its_own_refusal() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host_request);
    let frame = exported_frame(execution);
    let payload_len = (frame.bytes.len() - PAYLOAD_START) as u64;

    let mut flipped = frame.bytes.clone();
    *flipped.last_mut().unwrap() ^= 0x01;
    assert!(matches!(
        refusal(&flipped, execution),
        Refusal::Frame(FrameRefusal::DigestMismatch { .. })
    ));
    assert!(matches!(
        Artifact::from_untrusted_bytes(flipped),
        Err(FrameRefusal::DigestMismatch { .. })
    ));

    let mut wrong_magic = frame.bytes.clone();
    wrong_magic[MAGIC.start] ^= 0xff;
    assert!(matches!(
        refusal(&wrong_magic, execution),
        Refusal::Frame(FrameRefusal::UnrecognizedMagic)
    ));

    for unsupported_version in [1_u16, 2, 3, 4, 5, 6, 7, 9] {
        let mut old = frame.bytes.clone();
        old[VERSION].copy_from_slice(&unsupported_version.to_le_bytes());
        assert!(matches!(
            refusal(&old, execution),
            Refusal::Frame(FrameRefusal::UnsupportedVersion(reason))
                if reason.produced().get() == u32::from(unsupported_version)
        ));
    }

    let truncated_payload = &frame.bytes[..frame.bytes.len() - 1];
    assert!(matches!(
        refusal(truncated_payload, execution),
        Refusal::Frame(FrameRefusal::TruncatedPayload { declared, available })
            if declared == payload_len && available == payload_len - 1
    ));
    assert!(matches!(
        refusal(&frame.bytes[..COMMIT.end], execution),
        Refusal::Frame(FrameRefusal::TruncatedHeader { available }) if available == COMMIT.end
    ));

    // A MessagePack value inside the declared payload is a different defect
    // from extra bytes outside the frame. The three original value commitments
    // stay intact; the v8 grammar refuses the fourth value before decoding.
    let mut payload_tail = frame.bytes.clone();
    payload_tail.push(0xc0);
    write_u64(&mut payload_tail, LENGTH, payload_len + 1);
    assert!(matches!(
        refusal(&payload_tail, execution),
        Refusal::Frame(FrameRefusal::InvalidPayloadShape)
    ));

    let mut trailing = frame.bytes.clone();
    trailing.push(0);
    assert!(matches!(
        refusal(&trailing, execution),
        Refusal::Frame(FrameRefusal::TrailingBytes { .. })
    ));
}

#[test]
fn header_rewritten_with_a_recomputed_digest_disagrees_with_the_payload() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host_request);
    let frame = exported_frame(execution);

    let mut moved = frame.bytes.clone();
    write_u64(&mut moved, POSITION, frame.position.0 + 1);
    reseal(&mut moved);
    assert!(matches!(
        refusal(&moved, execution),
        Refusal::PositionDisagreement { header, payload }
            if header.0 == frame.position.0 + 1 && payload == frame.position
    ));

    let mut renamed = frame.bytes.clone();
    write_u64(&mut renamed, COMMIT, frame.commit_id.0 + 1);
    reseal(&mut renamed);
    assert!(matches!(
        refusal(&renamed, execution),
        Refusal::CommitIdDisagreement { header, envelope }
            if header.0 == frame.commit_id.0 + 1 && envelope == frame.commit_id
    ));
}

#[test]
fn intact_frames_with_foreign_payloads_refuse_at_decode_or_canonical_form() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host_request);
    let frame = exported_frame(execution);

    // The v8 commitment map remains valid, but its canonical owner is nil.
    let undecodable = rewrite_payload(
        &frame.bytes,
        &["position", "canonical", "original_inputs"],
        true,
    );
    assert!(matches!(
        refusal(&undecodable, execution),
        Refusal::PayloadDecode { .. }
    ));

    // Equal leaves do not authorize a different canonical map order.
    let padded = rewrite_payload(
        &frame.bytes,
        &["canonical", "position", "original_inputs"],
        false,
    );
    assert!(matches!(
        refusal(&padded, execution),
        Refusal::NonCanonicalPayload
    ));
}
