use super::*;
use std::mem::size_of;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn full_query_result_vector_admits_replacement_peak_before_growth() {
    let saved = ["saved-a", "saved-b"];
    let incoming_text = "incoming";
    let header = size_of::<super::super::QueryWorkerFragment>() as u64;
    let old_inline = (saved.len() * size_of::<String>()) as u64;
    let old_payload = saved.iter().map(|text| text.len() as u64).sum::<u64>();
    let new_inline = ((saved.len() + 1) * size_of::<String>()) as u64;
    let incoming_payload = incoming_text.len() as u64;
    let retained = old_inline + old_payload;
    let peak = retained + new_inline + incoming_payload;
    let final_heap = new_inline + old_payload + incoming_payload;
    assert!(
        final_heap < peak,
        "the final-only quote misses the full old backing"
    );
    for under in [true, false] {
        let ceiling = peak - u64::from(under);
        let lease = crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, Some(ceiling), None),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        let memory = QueryPacketMemory::tracked(Some(&lease)).unwrap();
        let mut result_bytes = header + retained;
        memory.result(result_bytes).unwrap();
        let mut records = Vec::with_capacity(saved.len());
        records.extend(saved.iter().map(|text| String::from(*text)));
        assert_eq!(
            records.len(),
            records.capacity(),
            "an already full, nonempty buffer"
        );
        let original = records.as_ptr();
        let saved_payloads = [records[0].as_ptr(), records[1].as_ptr()];
        let incoming = String::from(incoming_text);
        let result = reserve_record(&memory, &mut result_bytes, &mut records, &incoming);
        if under {
            assert!(matches!(
                result,
                Err(MapKernelFailure::Domain(
                    QueryReadPacketDenial::ReadRequest(
                        crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                            ancestor: 0,
                            ..
                        }
                    )
                ))
            ));
            assert_eq!(
                records.as_ptr(),
                original,
                "refusal precedes replacement and preserves the original buffer"
            );
            assert_eq!(records, saved);
            assert_eq!(result_bytes, header + retained);
            assert_eq!(lease.charged_memory_bytes(), retained);
        } else {
            result.unwrap();
            assert_ne!(
                records.as_ptr(),
                original,
                "replacement was allocated while the old vector still lived"
            );
            assert_eq!(
                records, saved,
                "move every existing row without cloning or reconstructing it"
            );
            assert_eq!(
                [records[0].as_ptr(), records[1].as_ptr()],
                saved_payloads,
                "preserve the original saved payload allocations"
            );
            assert_eq!(records.capacity(), saved.len() + 1);
            assert_eq!(result_bytes, header + final_heap);
            assert_eq!(
                lease.charged_memory_bytes(),
                final_heap,
                "old backing retired; saved and incoming payloads remain funded"
            );
            records.push(incoming);
            assert_eq!(records, ["saved-a", "saved-b", "incoming"]);
        }
        drop(records);
        drop(memory);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
