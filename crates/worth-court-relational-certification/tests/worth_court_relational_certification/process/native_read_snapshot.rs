//! Original read buffers stay funded until the final owned clone retires.
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionRequestArrayBuilder, SerialMemoryBudget,
    SerialRequest,
};

fn serial(limit: u64) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(limit)),
        CancellationSource::new().token(),
        None,
    )
}

fn retained_byte_owner_bill(bytes: &[u8]) -> u64 {
    let host = serial(1 << 20);
    let execution = ExecutionRequest::serial(&host);
    let mut building =
        ExecutionRequestArrayBuilder::<u8>::allocate(execution, bytes.len()).unwrap();
    building.extend_from_slice(bytes).unwrap();
    let owned = building.seal_immutable_bytes().unwrap();
    assert_eq!(owned.charged_payload_bytes(), Some(bytes.len() as u64));
    let memory = host.memory().unwrap();
    let total = memory.charged_memory_bytes();
    assert!(total > bytes.len() as u64);
    drop(owned);
    assert_eq!(memory.charged_memory_bytes(), 0);
    total
}

#[test]
fn memory_cut_keeps_original_buffers_and_refunds_after_last_owned_clone() {
    use worth_testkit_relational::durable_medium::{
        bytes::MemberBytes, OwnedNativeReadSnapshot, OwnedSnapshotDenial,
    };
    let host = serial(1 << 20);
    let request = ExecutionRequest::serial(&host);
    let mut original = vec![57u8; 57];
    let transition = [31u8; 31];
    let snapshot = OwnedNativeReadSnapshot::capture(
        &request,
        [(1, original.as_slice())].into_iter(),
        [(1, transition.as_slice())].into_iter(),
        &MemberBytes::default(),
        None,
    )
    .unwrap();
    assert_eq!(snapshot.bounds()[0], 1);
    assert_eq!(snapshot.bounds()[1], 1);
    original.fill(0);
    let owned = snapshot.canonical(&request, 1, 57).unwrap();
    assert_eq!(owned.bytes(), &[57u8; 57]);
    assert_eq!(
        snapshot.transition(&request, 1, 31).unwrap().bytes(),
        transition
    );
    assert!(matches!(
        snapshot.canonical(&request, 1, 56),
        Err(OwnedSnapshotDenial::Limit {
            length: 57,
            limit: 56
        })
    ));
    let clone = owned.clone();
    drop(owned);
    drop(snapshot);
    assert_eq!(
        host.memory().unwrap().charged_memory_bytes(),
        retained_byte_owner_bill(&[57u8; 57])
    );
    drop(clone);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    let zero = serial(0);
    let denied = ExecutionRequest::serial(&zero);
    assert!(matches!(
        OwnedNativeReadSnapshot::capture(
            &denied,
            [(1, original.as_slice())].into_iter(),
            std::iter::empty(),
            &MemberBytes::default(),
            None,
        ),
        Err(OwnedSnapshotDenial::Request(_))
    ));
    assert_eq!(zero.memory().unwrap().charged_memory_bytes(), 0);
}
