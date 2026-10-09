//! Concurrent port deliveries publish one exact serial transition chain.

use super::*;

#[test]
fn concurrent_port_deliveries_publish_one_exact_serial_transition_chain() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (_runtime, service, contract, source_owner) = fixture(None);
    let predecessor = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("source")),
        )
        .unwrap();
    execute(&service, &predecessor, 1, 7);
    let service = std::sync::Arc::new(service);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut completions = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let service = std::sync::Arc::clone(&service);
                let barrier = std::sync::Arc::clone(&barrier);
                let contract = contract.clone();
                scope.spawn(move || {
                    barrier.wait();
                    service
                        .deliver_committed_patch(
                            request_execution,
                            &contract,
                            SignalCommittedPatchDeliveryRequest::new([target(&contract)]),
                        )
                        .unwrap()
                })
            })
            .collect();
        barrier.wait();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    let first_order = service.readmit_evaluation(ReadmissionRequest {
        predecessor: &predecessor,
        transitions: &[
            completions[0].successor_transition(),
            completions[1].successor_transition(),
        ],
    });
    let successor = match first_order {
        Ok(successor) => successor,
        Err(ReadmissionDenial::TransitionChainMismatch) => {
            completions.swap(0, 1);
            service
                .readmit_evaluation(ReadmissionRequest {
                    predecessor: &predecessor,
                    transitions: &[
                        completions[0].successor_transition(),
                        completions[1].successor_transition(),
                    ],
                })
                .unwrap()
        }
        Err(denial) => panic!("unexpected concurrent delivery denial: {denial:?}"),
    };
    let (successor, counters) = successor.into_parts();
    assert_eq!(counters.transitions_checked(), 2);
    assert_eq!(
        execute(&service, &successor, 2, 8),
        SignalConditionalDecisionClass::ComputedChanged
    );
}
