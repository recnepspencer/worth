use super::*;
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequestArrayBuilder;

#[test]
fn protected_members_survive_replacement_while_ordinary_reads_follow_current_anchor() {
    let directory = tempfile::tempdir().unwrap();
    let serving = initialize(directory.path());
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        cancellation.token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let original = vec![51; 150_001];
    let original_key = Sha256::digest(&original).into();
    let first = serving
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                anchor(),
                0,
                b"first".to_vec(),
                fixture::placement(),
                deadline(),
            )
            .with_members(vec![
                AnchoredRecordMember::New {
                    key: original_key,
                    bytes: original.clone(),
                },
                AnchoredRecordMember::New {
                    key: original_key,
                    bytes: original.clone(),
                },
            ]),
        )
        .unwrap();
    assert_eq!(first.members()[0], first.members()[1]);
    let selected = serving
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                anchor(),
                1,
                b"original selected".to_vec(),
                fixture::placement(),
                deadline(),
            )
            .with_members(vec![
                AnchoredRecordMember::Carried {
                    locator: first.members()[0],
                },
                AnchoredRecordMember::New {
                    key: original_key,
                    bytes: original.clone(),
                },
                AnchoredRecordMember::New {
                    key: original_key,
                    bytes: original.clone(),
                },
            ]),
        )
        .unwrap();
    assert_ne!(selected.members()[0], selected.members()[1]);
    assert_eq!(selected.members()[1], selected.members()[2]);
    let reader = serving.try_records_for_checkpoint(&request).unwrap();
    let records = serving.anchored_records().unwrap();
    let membership = records
        .protected_membership_from(&reader, &request, anchor())
        .unwrap()
        .unwrap();
    assert_eq!(
        membership.member(original_key, original.len() as u64),
        Some(selected.members()[0])
    );
    let replacement = b"new actual member".to_vec();
    let latest = records
        .publish(
            AnchoredRecordPublication::new(
                anchor(),
                2,
                b"latest payload".to_vec(),
                fixture::placement(),
                deadline(),
            )
            .with_members(vec![AnchoredRecordMember::New {
                key: Sha256::digest(&replacement).into(),
                bytes: replacement.clone(),
            }]),
        )
        .unwrap();
    for locator in [selected.members()[0], selected.members()[1]] {
        let mut stream = records
            .open_protected_member(&reader, &membership, locator, 1 << 20)
            .unwrap();
        let mut bytes =
            ExecutionRequestArrayBuilder::<u8>::allocate(request, original.len()).unwrap();
        let mut scratch = [0; 16 * 1024];
        loop {
            request.check_live().unwrap();
            let count = stream.read_next(&mut scratch).unwrap();
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&scratch[..count]).unwrap();
        }
        let bytes = bytes.seal().unwrap().into_immutable_bytes();
        assert_eq!(bytes.bytes(), original);
        assert_eq!(bytes.charged_payload_bytes(), Some(original.len() as u64));
    }
    let current = serving.try_records_for_checkpoint(&request).unwrap();
    let before = serving.media_counters();
    assert!(matches!(
        records.open_protected_member(&current, &membership, selected.members()[0], 1 << 20),
        Err(AnchoredRecordMemberReadFailure::ForeignReader)
    ));
    assert_eq!(
        serving.media_counters(),
        before,
        "cross-root membership refuses before I/O"
    );
    let ordinary = records.read(anchor(), 1 << 20).unwrap().unwrap();
    assert_eq!(ordinary.version(), latest.version());
    assert_eq!(ordinary.bytes(), b"latest payload");
    assert_eq!(ordinary.members(), latest.members());
    assert_eq!(
        records
            .read_member(anchor(), latest.members()[0], 1 << 20)
            .unwrap(),
        replacement
    );
    assert!(matches!(
        records.read_member(anchor(), selected.members()[0], 1 << 20),
        Err(AnchoredRecordMemberReadFailure::NotCurrentMember)
    ));
    assert_eq!(
        records.read(anchor(), 1 << 20).unwrap().unwrap().version(),
        latest.version()
    );
    drop(current);
    drop(membership);
    drop(reader);
    serving.close();
}
