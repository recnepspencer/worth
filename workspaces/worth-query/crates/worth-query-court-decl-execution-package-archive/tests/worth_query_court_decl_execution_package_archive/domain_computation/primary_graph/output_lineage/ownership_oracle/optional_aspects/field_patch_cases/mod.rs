//! Real handler writes test the sealed-observation and installed-parent boundary.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::own_effect::SourceWrite;
macro_rules! schema {
    ($schema:ident, $owner:literal, $write:ident) => {
        worth_query_decl::facade::worth_query_application! {
            pub $schema { owner: $owner, version: (1, 0), contributions: [crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Members], }
        }
        impl TopologyModelBinding for $schema {}
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
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<UnobservedSchema>(false);
    let denial = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&runtime, true)
        .err()
        .expect("the invalid write must refuse");
    let WorthQueryOutputDemandDenialKind::UnobservedWrite(cause) = denial.kind() else {
        panic!("the original writer cause must survive producer delivery: {denial:?}");
    };
    assert_eq!(
        denial.commit_denial_kind(),
        None,
        "the writer refused before commit lowering"
    );
    assert_eq!(
        cause.locator(),
        (*(*(*(*runtime.runtime_for_test()).primary_provider_for_test()).graph_for_test())
            .layout_for_test())
        .field_locator_for_test("Body", "SourceNotes", "SourceContent")
        .unwrap()
    );
    assert_eq!(
        denial.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Terminal
    );
    assert_eq!(source_values(&runtime), (None, None));
}
#[test]
fn absent_optional_struct_initialization_requires_complete_required_fields() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<IncompleteSchema>(false);
    let denial = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&runtime, true)
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
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<PresentSchema>(false);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    runtime
        .with_application_advancement(&request, |phase| {
            crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::retained_recovery::materialize_source_in_phase(&runtime, &phase);
        })
        .unwrap();
    let before = source_values(&runtime);
    assert_eq!(before, (Some("complete-required-content".to_owned()), None));
    let settled = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&runtime, true).expect("observed present-parent patch");
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::readmit(&runtime, &settled, true);
    assert_eq!(
        source_values(&runtime),
        (before.0, Some("selected".to_owned()))
    );
}

pub fn source_values<Schema: BirthSchema>(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<Schema>,
) -> (Option<String>, Option<String>) {
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let external = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
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
                    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned(),
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
            (*runtime
                .mutation_projection_for_test())
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
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap()
}

#[path = "stale_initialization/mod.rs"]
pub mod stale_initialization;
