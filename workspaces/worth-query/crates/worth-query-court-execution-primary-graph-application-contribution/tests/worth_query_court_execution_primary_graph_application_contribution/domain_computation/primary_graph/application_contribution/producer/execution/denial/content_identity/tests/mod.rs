//! Real native controls at the new pre-effect storage receiver.
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Advance;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::dependency_identity::WorthQueryProducerIdentityDenial as Identity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::ProducerExecutionStop;
use worth_query_execution::facade::application_inspection::StoreDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::execution::denial::content_identity::identity_failed;
use worth_query_execution::facade::application_contribution::with_bootstrap_advancement;
use worth_execution::CancellationSource;
use worth_execution::ExecutionAllocationPolicy;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;

#[test]
fn actual_content_controls_and_resource_stops_keep_their_own_native_disposition() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 20), Some(1_000_000)),
    );
    with_bootstrap_advancement(policy, |phase| {
        let request = phase.execution_request();
        let ExecutionAllocationPolicy::Execution(lease) = request.allocation_policy() else {
            panic!("native payer");
        };
        for deadline in [false, true] {
            let source = CancellationSource::new();
            let child = lease.controlled_child(
                source.token(),
                deadline.then(|| {
                    let clock = worth_execution::MonotonicClock::system();
                    clock.deadline_at(clock.now())
                }),
            );
            if !deadline {
                source.cancel();
            }
            let refused = ExecutionAllocationPolicy::Execution(&child)
                .reserve_memory(1)
                .unwrap_err();
            let stop = identity_failed(
                "content",
                Identity::DependencyStorage(StoreDenial::Allocation(refused)),
            );
            let ProducerExecutionStop::RequestAdmissionDenied(rejected) = stop else {
                panic!("control must relinquish only this request");
            };
            assert_eq!(
                rejected.into_denial_for_test().kind(),
                if deadline {
                    WorthQueryOutputDemandDenialKind::TimedOut
                } else {
                    WorthQueryOutputDemandDenialKind::Cancelled
                }
            );
        }
        let blocked = lease
            .reserve_memory(
                lease
                    .available_memory_bytes()
                    .expect("this fixture supplies a finite memory ceiling")
                    - 1,
            )
            .unwrap();
        let refused = request.allocation_policy().reserve_memory(2).unwrap_err();
        let stop = identity_failed(
            "content",
            Identity::DependencyStorage(StoreDenial::Allocation(refused)),
        );
        let ProducerExecutionStop::ExecutionStopped(resource) = stop else {
            panic!("resource refusal is not an authority rejection");
        };
        assert!(matches!(
            resource.kind(),
            WorthQueryOutputDemandDenialKind::ExecutionRequest(Advance::Resource(_))
        ));
        drop(blocked);
        assert!(
            request.allocation_policy().reserve_memory(2).is_ok(),
            "fresh lawful native work remains possible"
        );
    })
    .unwrap();
}
