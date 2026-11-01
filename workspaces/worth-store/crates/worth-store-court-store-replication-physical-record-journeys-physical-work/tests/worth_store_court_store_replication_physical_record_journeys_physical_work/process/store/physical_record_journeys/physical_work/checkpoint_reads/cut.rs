use super::*;
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequestArrayBuilder;
#[test]
fn one_checkpoint_reader_preserves_anchor_and_three_journals_across_root_advance() {
    let directory = tempfile::tempdir().unwrap();
    let serving = initialize(directory.path());
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        cancellation.token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let original = vec![17; 150_001];
    publish_journals(&serving, 1, &original);
    let selected = publish_anchor(&serving, 0, original.clone());
    let reader = serving.try_records_for_checkpoint(&request).unwrap();
    publish_journals(&serving, 2, b"later journal payload");
    let later = publish_anchor(&serving, 1, b"later Home payload".to_vec());
    assert_ne!(selected.version(), later.version());
    let records = serving.anchored_records().unwrap();
    let mut stream = records
        .open_payload_from(&reader, &request, anchor(), 1 << 20)
        .unwrap()
        .unwrap();
    assert_eq!(
        stream.observed_generation(),
        selected.version().generation()
    );
    assert_eq!(stream.verified_version(), None);
    let mut output = ExecutionRequestArrayBuilder::<u8>::allocate(request, original.len()).unwrap();
    let mut scratch = [0; 100_000];
    loop {
        let count = stream.read_next(&mut scratch).unwrap();
        if count == 0 {
            break;
        }
        assert!(count <= 64 * 1024);
        assert_eq!(stream.verified_version(), None);
        output.extend_from_slice(&scratch[..count]).unwrap();
    }
    assert_eq!(stream.verified_version(), Some(selected.version()));
    let bytes = output.seal().unwrap().into_immutable_bytes();
    assert_eq!(bytes.bytes(), original);
    assert_eq!(bytes.charged_payload_bytes(), Some(original.len() as u64));
    let mut terminal_journal = None;
    for name in JOURNALS {
        let journal = serving
            .ordered_journal(OrderedJournalName::new(name).unwrap())
            .unwrap();
        assert_eq!(journal.head_position_from(&reader).unwrap(), 1);
        let described = journal.describe_entry_from(&reader, &request, 1).unwrap();
        assert_eq!(described.position(), 1);
        assert_eq!(described.payload_bytes(), original.len() as u64);
        assert_eq!(
            described.payload_sha256(),
            <[u8; 32]>::from(Sha256::digest(&original))
        );
        assert!(matches!(
            journal.describe_entry_from(&reader, &request, 2),
            Err(OrderedJournalReadFailure::NotFound { head: 1 })
        ));
        assert_eq!(journal.head().unwrap().unwrap().position(), 2);
        assert_eq!(journal.read(1, 1 << 20).unwrap().bytes(), original);
        assert_eq!(
            journal.read(2, 1 << 20).unwrap().bytes(),
            b"later journal payload"
        );
        assert!(matches!(
            journal.open_payload_from(&reader, &request, 2, 1 << 20),
            Err(OrderedJournalReadFailure::NotFound { head: 1 })
        ));
        let mut old = journal
            .open_payload_from(&reader, &request, 1, 1 << 20)
            .unwrap();
        let mut payload = Vec::new();
        loop {
            let count = old.read_next(&mut scratch).unwrap();
            if count == 0 {
                break;
            }
            assert!(count <= 64 * 1024);
            payload.extend_from_slice(&scratch[..count]);
        }
        assert_eq!(payload, original);
        let current = serving.try_records_for_checkpoint(&request).unwrap();
        assert_eq!(journal.head_position_from(&current).unwrap(), 2);
        let mut historical = journal
            .open_payload_from(&current, &request, 1, 1 << 20)
            .unwrap();
        let mut length = 0;
        while let count @ 1.. = historical.read_next(&mut scratch).unwrap() {
            length += count;
        }
        assert_eq!(length, original.len());
        terminal_journal = Some(old);
    }
    cancellation.cancel();
    assert_eq!(
        terminal_journal
            .as_mut()
            .unwrap()
            .read_next(&mut scratch)
            .unwrap(),
        0
    );
    drop(terminal_journal);
    assert_eq!(stream.read_next(&mut scratch).unwrap(), 0);
    assert_eq!(stream.verified_version(), Some(selected.version()));
    drop(stream);
    drop(reader);
    drop(bytes);
    serving.close();
}
