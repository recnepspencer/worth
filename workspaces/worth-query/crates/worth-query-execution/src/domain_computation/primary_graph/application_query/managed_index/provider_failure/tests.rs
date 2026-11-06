//! Local diagnostic values preserve type, equality and declared footprint.
//! Actual request retention is exercised by the public exact-refinement court.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::provider_failure::equal_cause;
use crate::domain_computation::vocabulary::primary_graph::provider_failure::PaidProviderDenial;
use crate::domain_computation::vocabulary::primary_graph::provider_failure::WorthQueryManagedIndexProviderFailure;
use worth_execution::ChargedBytes;
use {crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial, crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Domain(u8);
impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("same display")
    }
}
impl std::error::Error for Domain {}

// Private diagnostic fixture only: no request, Source, proof or storage grant.
fn retained(value: u8) -> WorthQueryManagedIndexProviderFailure {
    let wrapper = std::alloc::Layout::new::<PaidProviderDenial<Domain>>();
    let header = std::alloc::Layout::new::<[usize; 2]>();
    let bytes = header.extend(wrapper).unwrap().0.pad_to_align().size() as u64;
    WorthQueryManagedIndexProviderFailure {
        cause: Arc::new(PaidProviderDenial {
            cause: Domain(value),
            _memory: None,
        }),
        retained_bytes: bytes,
        equal: equal_cause::<Domain>,
    }
}

#[test]
fn realized_footprint_decision_failure_preserves_typed_provider_value_and_declared_bytes() {
    let original = retained(7);
    assert_eq!(original, retained(7));
    assert_ne!(original, retained(8)); // Equal formatting is not equal domain truth.
    let bytes = original.retained_payload_bytes();
    let failure = WorthQueryApplicationAttemptDenial::managed_index(
        WorthQueryManagedIndexWorkDenial::Provider(original),
        "target verification",
    )
    .into_decision_read_set_failure();
    let Some(WorthQueryManagedIndexWorkDenial::Provider(cause)) = failure.managed_index_denial()
    else {
        panic!("decision-read boundary discarded the provider cause")
    };
    assert_eq!(cause.downcast_ref::<Domain>(), Some(&Domain(7)));
    let cloned = failure.clone();
    assert_eq!(failure, cloned);
    let baseline = crate::domain_computation::WorthQueryDecisionReadSetFailure::new(
        failure.kind(),
        failure.detail(),
    );
    assert_eq!(
        failure.additional_charged_bytes(),
        baseline.additional_charged_bytes() + bytes
    );
    assert_eq!(
        cloned.managed_index_denial(),
        failure.managed_index_denial()
    );
}

#[test]
fn realized_footprint_decision_failure_preserves_native_execution_cause() {
    let original =
        worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled;
    let failure = WorthQueryApplicationAttemptDenial::relational_read_stopped(
        original,
        "target verification",
    )
    .into_decision_read_set_failure();
    assert!(matches!(failure.native_query_read_stop(), Some(
        worth_relational::facade::runtime::QueryReadExecutionStop::ReadRequest(cause)
    ) if *cause == original));
}
