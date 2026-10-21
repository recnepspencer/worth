//! Real handler writes test the sealed-observation and installed-parent boundary.
use super::*;
use own_effect::SourceWrite;
macro_rules! schema {
    ($schema:ident, $owner:literal, $write:ident) => {
        worth_query_decl::facade::worth_query_application! {
            pub(super) $schema { owner: $owner, version: (1, 0), contributions: [installation::Members], }
        }
        impl TopologySchemaBinding for $schema {}
        impl BirthSchema for $schema {
            const REQUIRED: bool = false;
            const SOURCE_WRITE: SourceWrite = SourceWrite::$write;
        }
    };
}
schema!(
    UnobservedSchema,
    "worth.query.test.optional-unobserved-write",
    Unobserved
);
schema!(
    IncompleteSchema,
    "worth.query.test.optional-incomplete-initialization",
    Incomplete
);
schema!(
    PresentSchema,
    "worth.query.test.optional-present-patch",
    Select
);

#[test]
fn absent_parent_write_requires_each_actual_target_observation() {
    let runtime = installation::install::<UnobservedSchema>(false);
    let denial = journey::demand(&runtime, true)
        .err()
        .expect("the invalid write must refuse");
    assert_eq!(
        denial.commit_denial_kind(),
        Some(WorthQueryApplicationCommitDenialKind::ProviderRejected)
    );
    let detail = format!("{denial:?}");
    assert!(
        detail.contains("ProposalBinding"),
        "unobserved Required Content must refuse at proposal binding: {detail}"
    );
    assert_eq!(source_values(&runtime), (None, None));
}
#[test]
fn absent_optional_struct_initialization_requires_complete_required_fields() {
    let runtime = installation::install::<IncompleteSchema>(false);
    let denial = journey::demand(&runtime, true)
        .err()
        .expect("the invalid write must refuse");
    assert_eq!(
        denial.commit_denial_kind(),
        Some(WorthQueryApplicationCommitDenialKind::ProviderRejected)
    );
    let detail = format!("{denial:?}");
    assert!(
        detail.contains("ProposalBinding"),
        "Selection alone cannot initialize Required Content: {detail}"
    );
    assert_eq!(source_values(&runtime), (None, None));
}
#[test]
fn present_parent_patch_preserves_unwritten_required_content() {
    let runtime = installation::install::<PresentSchema>(false);
    let request = primary_graph::tests::fixture::live_scope();
    runtime
        .with_application_advancement(&request, |phase| {
            retained_recovery::materialize_source_in_phase(&runtime, &phase);
        })
        .unwrap();
    let before = source_values(&runtime);
    assert_eq!(before, (Some("complete-required-content".to_owned()), None));
    let settled = journey::demand(&runtime, true).expect("observed present-parent patch");
    court::readmit(&runtime, &settled, true);
    assert_eq!(
        source_values(&runtime),
        (before.0, Some("selected".to_owned()))
    );
}

pub(super) fn source_values<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
) -> (Option<String>, Option<String>) {
    let request = primary_graph::tests::fixture::live_scope();
    let external = primary_graph::tests::fixture::authenticate_external(
        runtime.installed_schema(),
        "alice",
        std::time::Duration::from_secs(60),
        &request,
    );
    runtime
        .with_application_advancement(&request, |phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let binding = runtime
                .installed_schema()
                .principal_binding(ConsumerPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let source = selected
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    installation::ANCHOR.to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = runtime
                .installed_schema()
                .installed_operation(Produce::reference())
                .unwrap();
            let admitted = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &source,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            runtime
                .mutation_projection
                .project_admitted_operation(
                    &phase,
                    &admitted,
                    |reader, root| {
                        (
                            reader
                                .decision_field(root, SourceContent::reference())
                                .unwrap(),
                            reader
                                .decision_field(root, SourceSelection::reference())
                                .unwrap(),
                        )
                    },
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap()
}

mod stale_initialization;
