//! Each decoder call owns one bounded caller lease before entering its native phase.
use super::host::{Host, WorthQueryAdvancementPhase, WorthQueryRequestScope};
use worth_execution::{ExecutionAllocationPolicy, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

pub fn with_request<R>(
    host: &Host,
    scope: &WorthQueryRequestScope,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> R {
    let caller = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(64 * 1024 * 1024),
                Some(host.request_work),
            ),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .expect("the finite caller request is admitted");
    observe_decodes(host, || {
        host.program
            .runtime()
            .with_funded_application_advancement(
                scope,
                ExecutionAllocationPolicy::Execution(&caller),
                body,
            )
            .expect("the installed phase borrows the original funded caller")
    })
}

/// Observe the actual decoder boundary under either ordinary or native live funding.
pub(crate) fn observe_decodes<R>(host: &Host, body: impl FnOnce() -> R) -> R {
    super::opaque::begin_request();
    let observer =
        worth_execution::RequestConsultObservationForTest::install_on_this_thread_for_test(
            super::opaque::consult,
        );
    let answer = body();
    // Pick the real accepted prefix at the second seed's first charge. The
    // negative twin below must independently reach and swallow its next refusal.
    let second = super::opaque::second_decode_consult();
    host.second_decode_work.set(second.and_then(|consult| {
        observer
            .charged_prefixes_for_test()
            .into_iter()
            .filter(|(ordinal, _)| *ordinal <= consult)
            .map(|(_, work)| work)
            .max()
    }));
    answer
}
