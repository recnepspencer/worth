use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::direct_attempt;

use worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceAdmissionCounters;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot;
use worth_query_admission::facade::resource_admission::WorthQueryFixedExecutionCapacity;
use worth_query_admission::integration::admit_execution_resource_plan;
use worth_query_admission::integration::reserve_execution_resource_plan;
use worth_query_declaration::facade::domain_computation::WorthQueryCancellationSafePointFamily;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionMode;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest;
use worth_query_declaration::facade::domain_computation::WorthQueryPartialEffectPosture;
use worth_query_declaration::facade::domain_computation::WorthQueryResourceDimension;
use worth_query_declaration::facade::domain_computation::WorthQueryResourceLimitRequest;
use worth_query_declaration::facade::domain_computation::WorthQueryRetainedProgressPosture;
use worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleRequest;
use worth_query_declaration::facade::domain_computation::WorthQueryYieldedStatePosture;
use worth_query_installation::facade::WorthQueryExecutionAccessProductFamily;
use worth_query_installation::facade::WorthQueryExecutionAllocatorFamily;
use worth_query_installation::facade::WorthQueryExecutionProviderFamily;
use worth_query_installation::facade::WorthQueryExecutionProviderRequirements;
use worth_query_installation::facade::WorthQueryExecutionResourceContract;
use worth_query_installation::facade::WorthQueryExecutionResourceEnvelope;
use worth_query_installation::facade::WorthQueryExecutionStrategyContract;
use worth_query_installation::facade::WorthQueryExecutionStrategyName;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_execution::facade::domain_computation::WorthQueryDirectExecutionResourceAttempt;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority;
#[test]
fn direct_attempt_mints_one_session_and_binds_immutable_evidence() {
    let attempt = direct_attempt("direct", 8);

    assert_eq!(attempt.resources().counters().provider_session_mints, 1);
    assert_eq!(
        attempt.provider_session_for_test().attempt_identity(),
        attempt.attempt_identity().as_str()
    );
    assert_eq!(
        attempt.evidence().admission_identity(),
        attempt.resources().identity()
    );
    assert_eq!(
        attempt.evidence().provider_session_identity(),
        attempt.provider_session_for_test().identity()
    );
    assert_eq!(
        attempt.evidence().provider_session_attempt_identity(),
        attempt.provider_session_for_test().attempt_identity()
    );
}
#[test]
fn repeated_direct_attempts_receive_unique_provider_sessions() {
    let first = direct_attempt("repeat", 8);
    let second = direct_attempt("repeat", 8);

    assert_ne!(
        first.provider_session_for_test().attempt_identity(),
        second.provider_session_for_test().attempt_identity()
    );
    assert_eq!(
        first.provider_session_for_test().attempt_identity(),
        first.attempt_identity().as_str()
    );
    assert_eq!(
        second.provider_session_for_test().attempt_identity(),
        second.attempt_identity().as_str()
    );
    assert_ne!(
        first.provider_session_for_test().identity(),
        second.provider_session_for_test().identity()
    );
    assert_ne!(first.evidence().identity(), second.evidence().identity());
}
