use super::fixtures::{
    certification_authority_source_min_one_runtime, publication_pair_min_two_runtime,
    publication_source_min_one_runtime,
};
use crate::facade::publication::PublicationError;
use crate::facade::runtime::InvariantExecutionResult;
use crate::tests::support::*;

#[test]
fn relation_integrity_certification_boundary_rejects_zero_edge_entity_for_minimum_cardinality() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = publication_source_min_one_runtime();
    let _orphan = create_entity(execution, &runtime, "orphan");

    let result = runtime.validation().certification_state();
    let failure = result
        .summary()
        .publication_failure()
        .expect("certification minimum cardinality failure");

    assert_eq!(failure.code(), DiagnosticCode::RelationCardinalityViolation);
    match failure.fields() {
        crate::validation::data::InvariantViolationFields::RelationCardinalityEndpoint {
            contract_id,
            relation_kind_id,
            boundary,
            count,
            limit,
            ..
        } => {
            assert_eq!(contract_id.as_str(), "source_min_one");
            assert_eq!(*relation_kind_id, KindId(2));
            assert_eq!(
                *boundary,
                crate::validation::data::RelationCardinalityBoundary::Source
            );
            assert_eq!(*count, 0);
            assert_eq!(*limit, 1);
        }
        fields => panic!("expected typed cardinality endpoint fields, got {fields:?}"),
    }
    assert_eq!(
        result.metadata().execution_point().diagnostic_label(),
        "certification_boundary"
    );
}

#[test]
fn relation_integrity_certification_boundary_rejects_observed_pair_below_parallel_minimum() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = publication_pair_min_two_runtime();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    create_relation(execution, &runtime, source, target, "single");

    let result = runtime.validation().certification_state();
    let failure = result
        .summary()
        .publication_failure()
        .expect("certification pair minimum failure");

    assert_eq!(failure.code(), DiagnosticCode::RelationCardinalityViolation);
    match failure.fields() {
        crate::validation::data::InvariantViolationFields::RelationCardinalityPair {
            contract_id,
            relation_kind_id,
            source: actual_source,
            target: actual_target,
            count,
            limit,
        } => {
            assert_eq!(contract_id.as_str(), "pair_min_two");
            assert_eq!(*relation_kind_id, KindId(2));
            assert_eq!(
                *actual_source,
                crate::transactions::data::EntityReference::Existing(source)
            );
            assert_eq!(
                *actual_target,
                crate::transactions::data::EntityReference::Existing(target)
            );
            assert_eq!(*count, 1);
            assert_eq!(*limit, 2);
        }
        fields => panic!("expected typed cardinality pair fields, got {fields:?}"),
    }
}

#[test]
fn relation_integrity_certification_boundary_is_authority_owned_and_blocks_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = certification_authority_source_min_one_runtime();
    let _orphan = create_entity(execution, &runtime, "orphan");

    let shared: &RelationalRuntime = &runtime;
    let error = shared
        .certify_current_state()
        .expect_err("certification boundary should block incomplete topology");

    assert_eq!(error.stage, PublicationStage::InvariantCheck);
    assert!(error.detail.contains("source_min_one"));
}

/// The certification boundary observes state and emits diagnostics through
/// shared capabilities, so it belongs to the shared-borrow receiver matrix.
/// Coercing it to a shared-receiver function pointer fails to compile the moment
/// it reclaims an exclusive borrow of the whole runtime.
#[test]
fn certification_boundary_is_addressable_through_a_shared_borrow() {
    let _certify: fn(&RelationalRuntime) -> Result<InvariantExecutionResult, PublicationError> =
        RelationalRuntime::certify_current_state;
}
