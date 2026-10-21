//! Required field absence under an explicitly Optional whole aspect.
use super::*;
use installation::OptionalSchema;
use std::time::Duration;

pub(super) fn assert_absent_required_content_is_readable(
    runtime: &installation::Application<OptionalSchema>,
) {
    let request = primary_graph::tests::fixture::live_scope();
    let external = primary_graph::tests::fixture::authenticate_external(
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
        let source = selected.resolve_entity(&phase, BodyKey::reference(), installation::ANCHOR.to_owned(), &request,
            WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
        let operation = runtime.installed_schema().installed_operation(Produce::reference()).unwrap();
        let admitted = selected.authorize_operation(&phase, &principal, &source, &operation, Default::default(), &request).unwrap();
        let (_, snapshot, _) = runtime.mutation_projection.project_admitted_operation(&phase, &admitted, |reader, root| {
            assert_eq!(reader.decision_field(root, SourceContent::reference()).unwrap(), None);
            let called = std::cell::Cell::new(false);
            assert_eq!(reader.decision_field_with_predecode_admission(root, SourceContent::reference(), |_, _| {
                called.set(true); Ok::<_, ()>(())
            }, &|| Ok(())).unwrap().unwrap(), None);
            assert!(!called.get());
        }, crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation).unwrap().into_parts();
        let reads = runtime.begin_projected_application_read_attempt(&phase, admitted, snapshot,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation).unwrap()
            .complete_projected_dependencies(&phase, crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation).unwrap();
        assert!(reads.decision_facts().iter().any(|fact| matches!(fact,
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::OptionalAspectAbsent { .. })),
            "Required content can be sealed as whole-aspect absence only under the installed Optional law");
    }).unwrap();
}
