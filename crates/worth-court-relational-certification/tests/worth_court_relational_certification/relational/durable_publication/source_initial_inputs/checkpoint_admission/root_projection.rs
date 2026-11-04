//! Rehashed DATA must still obey the declared route-selector grammar.
use super::*;

pub(super) fn root_payload(raw: &[u8]) -> &[u8] {
    let mut at = 64 + u32::from_le_bytes(raw[12..16].try_into().unwrap()) as usize;
    let (tag, payload) = references::group(raw, &mut at);
    assert_eq!(tag, 9);
    payload
}

pub(super) fn replace_root(raw: &[u8], root: &[u8]) -> Vec<u8> {
    let cut_end = 64 + u32::from_le_bytes(raw[12..16].try_into().unwrap()) as usize;
    let cut = &raw[64..cut_end];
    let previous_length = u64::from_le_bytes(raw[24..32].try_into().unwrap()) as usize;
    let tail_start = cut_end + 41 + previous_length;
    let mut meaning = Sha256::new();
    meaning.update(b"worth.relational.checkpoint-admission.meaning.v1\0");
    meaning.update(1_u32.to_le_bytes());
    meaning.update((cut.len() as u64).to_le_bytes());
    meaning.update(cut);
    meaning.update(0_u64.to_le_bytes());
    meaning.update(0_u64.to_le_bytes());
    meaning.update([9]);
    meaning.update((root.len() as u64).to_le_bytes());
    meaning.update(root);
    let mut root_header = [0_u8; 41];
    root_header[0] = 9;
    root_header[1..9].copy_from_slice(&(root.len() as u64).to_le_bytes());
    root_header[9..].copy_from_slice(&meaning.finalize());
    let mut header = raw[..64].to_vec();
    header[24..32].copy_from_slice(&(root.len() as u64).to_le_bytes());
    let mut directory = Sha256::new();
    directory.update(b"worth.relational.checkpoint-admission.directory.v1\0");
    directory.update(&header[..32]);
    directory.update(cut);
    directory.update(root_header);
    let mut at = tail_start;
    while at < raw.len() {
        directory.update(&raw[at..at + 16]);
        at += 16;
        for _ in 0..8 {
            directory.update(&raw[at..at + 41]);
            let length = u64::from_le_bytes(raw[at + 1..at + 9].try_into().unwrap()) as usize;
            at += 41 + length;
        }
    }
    header[32..].copy_from_slice(&directory.finalize());
    let mut output = header;
    output.extend_from_slice(cut);
    output.extend_from_slice(&root_header);
    output.extend_from_slice(root);
    output.extend_from_slice(&raw[tail_start..]);
    output
}

#[test]
fn checkpoint_image_refuses_unresolved_routes_outside_its_canonical_roster() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let captured = captured(request);
    let image = captured
        .admission_image()
        .seal_home_cut(&cut(&captured), request)
        .unwrap();
    let raw = image.as_bytes();
    let root = root_payload(raw);
    let key = b"unresolved_prefix";
    let offset = root
        .windows(key.len())
        .position(|value| value == key)
        .unwrap()
        + key.len();
    let original: serde_json::Value = crate::source_wire_value::read(root);
    assert!(original["roots"][0][1]["unresolved_prefix"].is_null());
    assert_eq!(root[offset], 0xc0);
    let mut invalid = root[..offset].to_vec();
    invalid.extend_from_slice(&rmp_serde::to_vec(&u64::MAX).unwrap());
    invalid.extend_from_slice(&root[offset + 1..]);
    assert_eq!(
        parse(&replace_root(raw, &invalid), request).unwrap_err(),
        Refusal::InvalidProjectedMeaning
    );
}

#[test]
fn checkpoint_image_seal_work_is_independent_of_digest_octet_values() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let captured = captured(request);
    let original = cut(&captured);
    let seal = |value| {
        let mut scope = *original.scope();
        let CheckpointAdmissionScope::Home {
            world_image_digest, ..
        } = &mut scope
        else {
            unreachable!()
        };
        *world_image_digest = [value; 32];
        let cut = CheckpointAdmissionCut::new(
            scope,
            original.canonical_through(),
            original.transitions_through(),
            *original.checkpoint_digest(),
        );
        let (image, report) = request
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                captured
                    .admission_image()
                    .seal_home_cut(&cut, request)
                    .unwrap()
            })
            .unwrap();
        (image.as_bytes().len(), report.charged_work())
    };
    assert_eq!(seal(0), seal(255));
}
