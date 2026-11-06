//! Required field absence under an explicitly Optional whole aspect.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::OptionalSchema;
use std::time::Duration;

pub fn assert_absent_required_content_is_readable(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<OptionalSchema>,
) {
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let external = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
        runtime.installed_schema(),
        "alice",
        Duration::from_secs(60),
        &request,
    );
    runtime.with_application_advancement(&request, |phase| {
        let selected = runtime.on_branch(runtime.current_world()).select(&phase).unwrap();
        let binding = runtime.installed_schema().principal_binding(ConsumerPrincipalBinding::reference()).unwrap();
        let principal = selected.resolve_authenticated_principal(&phase, &binding, &external, &request,
            WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
        let source = selected.resolve_entity(&phase, BodyKey::reference(), crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned(), &request,
            WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
        let operation = runtime.installed_schema().installed_operation(Produce::reference()).unwrap();
        let admitted = selected.authorize_operation(&phase, &principal, &source, &operation, Default::default(), &request).unwrap();
        let (_, snapshot, _) = (*runtime.mutation_projection_for_test()).project_admitted_operation(&phase, &admitted, |reader, root| {
            assert_eq!(reader.decision_field(root, SourceContent::reference()).unwrap(), None);
            let called = std::cell::Cell::new(false);
            assert_eq!(reader.decision_field_with_predecode_admission_for_test(root, SourceContent::reference(), |_, _| {
                called.set(true); Ok::<_, ()>(())
            }, &|| Ok(())).unwrap().unwrap(), None);
            assert!(!called.get());
        }, worth_execution::ExecutionAllocationPolicy::SystemAllocation).unwrap().into_parts();
        let reads = runtime.begin_projected_application_read_attempt(&phase, admitted, snapshot,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation).unwrap()
            .complete_projected_dependencies(&phase, worth_execution::ExecutionAllocationPolicy::SystemAllocation).unwrap();
        assert!(reads.decision_facts().iter().any(|fact| matches!(fact,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact::OptionalAspectAbsent { .. })),
            "Required content can be sealed as whole-aspect absence only under the installed Optional law");
        let control = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(
            worth_execution::ExecutionAllocationPolicy::SystemAllocation, None);
        let original = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload::copy_funded_for_test(
            &reads.decision_facts(), control).unwrap();
        let captured = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload::copy_performed_field_reads_for_test(
            &original, control).unwrap();
        drop(original);
        assert!(captured.iter_for_test().any(|fact| matches!(fact,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact::OptionalAspectAbsent { .. })),
            "genuine completed whole-aspect absence survives field-only custody capture");
    }).unwrap();
}
