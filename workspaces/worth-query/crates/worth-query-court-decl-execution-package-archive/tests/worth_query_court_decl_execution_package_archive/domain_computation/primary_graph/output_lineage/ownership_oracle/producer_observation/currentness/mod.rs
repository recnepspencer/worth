//! A genuine native lineage read refuses an absent retained-fact coordinate.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::producer_observation::*;
use worth_query_execution::facade::application_inspection::IndexedSelectionReobserveDenial as Indexed;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason as Requirement;
use worth_query_execution::facade::application_inspection::RebaseVerificationReason as Reason;

#[test]
fn retained_currentness_refuses_corrupted_fact_coordinates_before_clean_marks() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::install();
    let original = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::journey::complete(&runtime);
    let receipt = original.application_commit_receipt().unwrap();
    runtime.with_host_advancement(|phase| {
        let execution = phase.execution_request_for_for_test(&(*(*runtime.runtime_for_test()).product_runtime_for_test())).unwrap();
        let selected = runtime.on_branch(runtime.current_world()).select(&phase).unwrap();
        let mut read = {
            let lineage = (*(*(*runtime.runtime_for_test()).primary_provider_for_test()).graph_for_test()).output_lineage.lock().unwrap();
            lineage.source_facts_for_receipt_for_test((*runtime.runtime_for_test()).authority_identity_for_test().as_u64(),
                &(*runtime.installed_schema_for_test()).binding_identity(), selected.product().observation_for_test(), receipt, &execution).unwrap().unwrap()
        };
        assert!(!(*read.facts_for_test()).as_ref().unwrap().is_empty_for_test(), "real retained producer facts");
        (*(*(*runtime.runtime_for_test()).primary_provider_for_test()).graph_for_test()).with_runtime(&execution, |native| {
            selected.require_current_read_for_test(&execution, native, &read, Some(receipt)).unwrap();
            for (valid, requirement, expected) in [
                (Requirement::NativeFactRevisionUnavailable(0), Requirement::NativeFactRevisionUnavailable(usize::MAX), Reason::NativeFactRevisionUnavailable),
                (Requirement::IndexedSelectionFactDenied(0, Indexed::Overflow), Requirement::IndexedSelectionFactDenied(usize::MAX, Indexed::Overflow), Reason::IndexedSelectionFactDenied(Indexed::Overflow)),
            ] {
                (*read.verification_requirement_mut_for_test()) = Some(valid);
                selected.require_current_read_for_test(&execution, native, &read, Some(receipt)).expect("present retained coordinate in the genuine fact body");
                (*read.verification_requirement_mut_for_test()) = Some(requirement);
                let cause = selected.require_current_read_for_test(&execution, native, &read, Some(receipt)).expect_err("absent retained coordinate");
                assert_eq!(cause.kind(), worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind::RetainedVerificationFactUnavailable(expected));
            }
        });
    }).unwrap();
}
