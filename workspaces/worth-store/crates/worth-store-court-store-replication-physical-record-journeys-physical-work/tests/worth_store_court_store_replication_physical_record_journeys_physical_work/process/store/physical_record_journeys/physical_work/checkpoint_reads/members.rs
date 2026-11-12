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
    publish_journals(&serving, 1, b"selected journal payload");
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
    let read_charge = serial.memory().unwrap().charged_memory_bytes() - before_stream;
    assert!(
        read_charge > 0,
        "the original table and fixed future Arc have actual request custody"
    );
    let (mut stream, reader) = match stream.into_verified_membership(reader) {
        Err(same) => same,
        Ok(_) => panic!("membership cannot escape before verified EOF"),
    };
    assert_eq!(
        serial.memory().unwrap().charged_memory_bytes(),
        before_stream + read_charge
    );
    publish_journals(&serving, 2, b"later journal payload");
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
    let current = serving.try_records_for_checkpoint(&request).unwrap();
    let current_root = current.protected_root();
    let (stream, current) = match stream.into_verified_membership(current) {
        Err(same) => same,
        Ok(_) => panic!("a newer reader cannot replace the original read route"),
    };
    assert_eq!(stream.verified_version(), Some(selected.version()));
    assert_eq!(current.protected_root(), current_root);
    assert_eq!(
        serial.memory().unwrap().charged_memory_bytes(),
        charge_before_conversion
    );
    assert_eq!(serving.media_counters(), io_before_conversion);
    let owned = stream
        .into_verified_membership(reader)
        .unwrap_or_else(|_| panic!("actual verified EOF must move both original owners"));
    assert_eq!(
        serial.memory().unwrap().charged_memory_bytes(),
        charge_before_conversion
    );
    assert_eq!(
        records.validate_read_membership(&owned).unwrap(),
        selected.membership()
    );
    fn require_shared<T: Send + Sync + 'static>() {}
    require_shared::<AnchoredRecordReadMembership>();
    let creator = owned;
    let read_membership = creator.clone();
    drop(creator);
    assert_eq!(
        serial.memory().unwrap().charged_memory_bytes(),
        charge_before_conversion
    );
    assert_eq!(serving.media_counters(), io_before_conversion);
    assert_eq!(
        records.validate_read_membership(&read_membership).unwrap(),
        selected.membership()
    );
    for name in JOURNALS {
        let journal = serving
            .ordered_journal(OrderedJournalName::new(name).unwrap())
            .unwrap();
        assert_eq!(
            journal.head_position_from_read(&read_membership).unwrap(),
            1
        );
        let described = journal
            .describe_entry_from_read(&read_membership, &request, 1)
            .unwrap();
        assert_eq!(
            described.payload_bytes(),
            b"selected journal payload".len() as u64
        );
        assert!(matches!(
            journal.describe_entry_from_read(&read_membership, &request, 2),
            Err(OrderedJournalReadFailure::NotFound { head: 1 })
        ));
        let mut payload = journal
            .open_payload_from_read(&read_membership, &request, 1, 1 << 20)
            .unwrap();
        let mut output = Vec::new();
        let mut scratch = [0; 7];
        loop {
            let count = payload.read_next(&mut scratch).unwrap();
            if count == 0 {
                break;
            }
            output.extend_from_slice(&scratch[..count]);
        }
        assert_eq!(output, b"selected journal payload");
    }
    // Both distinct locators with equal content remain selectable by their
    // actual table positions after the Home anchor has replaced them.
    for table_index in [0, 1] {
        let mut member = records
            .open_read_member(&read_membership, table_index, 1 << 20)
            .unwrap();
        let mut bytes =
            ExecutionRequestArrayBuilder::<u8>::allocate(request, original.len()).unwrap();
        let mut scratch = [0; 16 * 1024];
        loop {
            request.check_live().unwrap();
            let count = member.read_next(&mut scratch).unwrap();
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&scratch[..count]).unwrap();
        }
        assert_eq!(
            bytes.seal().unwrap().into_immutable_bytes().bytes(),
            original
        );
    }
    let before = serving.media_counters();
    assert!(matches!(
        records.open_protected_member(&current, &membership, selected.members()[0], 1 << 20),
        Err(AnchoredRecordMemberReadFailure::ForeignReader)
    ));
    assert!(matches!(
        records.open_read_member(&read_membership, selected.members().len(), 1 << 20),
        Err(AnchoredRecordMemberReadFailure::NotCurrentMember)
    ));
    assert_eq!(
        serving.media_counters(),
        before,
        "index/root refusal precedes I/O"
    );
    let foreign_directory = tempfile::tempdir().unwrap();
    let foreign = initialize(foreign_directory.path());
    let before_foreign = foreign.media_counters();
    assert!(matches!(
        foreign
            .anchored_records()
            .unwrap()
            .validate_read_membership(&read_membership),
        Err(AnchoredRecordReadFailure::ForeignReader)
    ));
    assert_eq!(foreign.media_counters(), before_foreign);
    foreign.close();
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
        before_drop - read_charge
    );
    drop(current);
    drop(membership);
    serving.close();
}
