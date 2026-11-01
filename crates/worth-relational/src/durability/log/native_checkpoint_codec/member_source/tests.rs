//! Inline image bytes may be externally owned; owned outputs are newly admitted.
use super::*;
use crate::durability::medium::{
    RelationalDurableOwnedReadRefusal as Refusal, RelationalDurableReadProtection,
};
use sha2::{Digest, Sha256};
use worth_execution::{
    CancellationSource, ExecutionImmutableBytes, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
// Observe the public controller in a separate genuine finite fixture call.
fn controller_memory() -> u64 {
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut observed = None;
    request
        .run_native_with_checkpoint(|_| observed = Some(budget.charged_memory_bytes()))
        .unwrap();
    let observed = observed.expect("actual public controller entered");
    assert!(observed > 0);
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed
}
#[test]
fn inline_member_read_funds_output_and_keeps_grant_until_last_clone() {
    let bytes = vec![17u8; 57];
    let locator = RelationalDurableMemberLocator {
        sha256: Sha256::digest(&bytes).into(),
        length: bytes.len() as u64,
    };
    let imported =
        ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.clone().into_boxed_slice()));
    assert_eq!(imported.charged_payload_bytes(), None);
    let source: Arc<dyn RelationalDurableColdSource> = Arc::new(InlineCheckpointSource {
        members: BTreeMap::from([(locator, imported)]),
        envelopes: BTreeMap::new(),
    });
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&host);
    let cut = RelationalDurableReadProtection::capture(&source, &request).unwrap();
    assert_eq!(cut.bounds().canonical.0, 0);
    assert_eq!(cut.bounds().transitions.0, 0);
    let output = cut
        .read_member_owned_at(&source, &request, locator, 57)
        .unwrap();
    assert_eq!(output.bytes(), bytes);
    let independent_bill = std::alloc::Layout::array::<u8>(57).unwrap().size() as u64;
    assert_eq!(output.charged_payload_bytes(), Some(independent_bill));
    assert_eq!(
        host.memory().unwrap().charged_memory_bytes(),
        independent_bill
    );
    let clone = output.clone();
    drop(output);
    drop(cut);
    drop(source);
    assert_eq!(
        host.memory().unwrap().charged_memory_bytes(),
        independent_bill
    );
    drop(clone);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    let source: Arc<dyn RelationalDurableColdSource> = Arc::new(InlineCheckpointSource {
        members: BTreeMap::from([(
            locator,
            ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.into_boxed_slice())),
        )]),
        envelopes: BTreeMap::new(),
    });
    let small = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(
            controller_memory().checked_add(independent_bill).unwrap() - 1,
        )),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&small);
    let cut = RelationalDurableReadProtection::capture(&source, &request).unwrap();
    // The genuine controller fits; only the original output reservation is refused.
    let denial = cut
        .read_member_owned_at(&source, &request, locator, 57)
        .unwrap_err();
    let requested = match denial {
        Refusal::Buffer(worth_execution::ExecutionRequestArrayDenial::Request(
            worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(
                    worth_execution::MemoryLimitDenial::Capacity { requested, .. },
                ),
            ),
        )) => requested,
        other => panic!("exact entered output allocation required: {other:?}"),
    };
    assert_eq!(requested, independent_bill);

    assert_eq!(small.memory().unwrap().charged_memory_bytes(), 0);
}
