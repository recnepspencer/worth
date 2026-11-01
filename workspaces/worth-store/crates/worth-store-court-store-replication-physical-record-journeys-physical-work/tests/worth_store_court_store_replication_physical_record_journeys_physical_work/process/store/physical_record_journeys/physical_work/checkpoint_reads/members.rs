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
    let before_stream = serial.memory().unwrap().charged_memory_bytes();
    let stream = records
        .open_payload_from(&reader, &request, anchor(), 1 << 20)
        .unwrap()
        .unwrap();
    let table_charge = serial.memory().unwrap().charged_memory_bytes() - before_stream;
    assert!(
        table_charge > 0,
        "the original table has actual request custody"
    );
    let mut stream = match stream.into_verified_membership() {
        Err(same) => same,
        Ok(_) => panic!("membership cannot escape before verified EOF"),
    };
    assert_eq!(
        serial.memory().unwrap().charged_memory_bytes(),
        before_stream + table_charge
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
    let mut selected_payload = Vec::new();
    let mut scratch = [0; 7];
    loop {
        let count = stream.read_next(&mut scratch).unwrap();
        if count == 0 {
            break;
        }
        selected_payload.extend_from_slice(&scratch[..count]);
    }
    assert_eq!(selected_payload, b"original selected");
    let charge_before_conversion = serial.memory().unwrap().charged_memory_bytes();
    let io_before_conversion = serving.media_counters();
    let read_membership = stream
        .into_verified_membership()
        .unwrap_or_else(|_| panic!("actual verified EOF must move the original table"));
    assert_eq!(
        serial.memory().unwrap().charged_memory_bytes(),
        charge_before_conversion
    );
    assert_eq!(serving.media_counters(), io_before_conversion);
    assert_eq!(
        records
            .validate_read_membership(&reader, &read_membership)
            .unwrap(),
        selected.membership()
    );
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
    assert!(matches!(
        records.validate_read_membership(&current, &read_membership),
        Err(AnchoredRecordReadFailure::ForeignReader)
    ));
    assert_eq!(serving.media_counters(), before);
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
    // Explicit ordinary disposal is outside the conversion's bounded contract.
    let before_drop = serial.memory().unwrap().charged_memory_bytes();
    drop(read_membership);
    assert_eq!(
        serial.memory().unwrap().charged_memory_bytes(),
        before_drop - table_charge
    );
    drop(current);
    drop(membership);
    drop(reader);
    serving.close();
}
