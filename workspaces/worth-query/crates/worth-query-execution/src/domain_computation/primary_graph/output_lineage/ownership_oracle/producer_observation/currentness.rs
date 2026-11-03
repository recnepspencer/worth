//! A genuine native lineage read refuses an absent retained-fact coordinate.
use super::*;
use primary_graph::application_attempt::IndexedSelectionReobserveDenial as Indexed;
use primary_graph::output_lineage::invalidation::FullVerificationReason as Requirement;
use primary_graph::provider::RebaseVerificationReason as Reason;

#[test]
fn retained_currentness_refuses_corrupted_fact_coordinates_before_clean_marks() {
    let runtime = installation::install();
    let original = journey::complete(&runtime);
    let receipt = original.application_commit_receipt().unwrap();
    runtime.with_host_advancement(|phase| {
        let execution = phase.execution_request_for(&runtime.runtime.product_runtime).unwrap();
        let selected = runtime.on_branch(runtime.current_world()).select(&phase).unwrap();
        let mut read = {
            let lineage = runtime.runtime.primary_provider.graph.output_lineage.lock().unwrap();
            lineage.source_facts_for_receipt(runtime.runtime.authority_identity().as_u64(),
                &runtime.installed_schema.binding_identity(), selected.product().observation(), receipt, &execution).unwrap().unwrap()
        };
        assert!(!read.facts.as_ref().unwrap().is_empty(), "real retained producer facts");
        runtime.runtime.primary_provider.graph.with_runtime(&execution, |native| {
            selected.require_current_read(&execution, native, &read, Some(receipt)).unwrap();
            for (valid, requirement, expected) in [
                (Requirement::NativeFactRevisionUnavailable(0), Requirement::NativeFactRevisionUnavailable(usize::MAX), Reason::NativeFactRevisionUnavailable),
                (Requirement::IndexedSelectionFactDenied(0, Indexed::Overflow), Requirement::IndexedSelectionFactDenied(usize::MAX, Indexed::Overflow), Reason::IndexedSelectionFactDenied(Indexed::Overflow)),
            ] {
                read.verification_requirement = Some(valid);
                selected.require_current_read(&execution, native, &read, Some(receipt)).expect("present retained coordinate in the genuine fact body");
                read.verification_requirement = Some(requirement);
                let cause = selected.require_current_read(&execution, native, &read, Some(receipt)).expect_err("absent retained coordinate");
                assert_eq!(cause.kind(), primary_graph::WorthQueryOutputDemandDenialKind::RetainedVerificationFactUnavailable(expected));
            }
        });
    }).unwrap();
}
