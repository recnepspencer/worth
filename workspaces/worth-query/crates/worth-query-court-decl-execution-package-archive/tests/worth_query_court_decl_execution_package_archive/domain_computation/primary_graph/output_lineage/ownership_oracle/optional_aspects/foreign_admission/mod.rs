//! A foreign admission carrying source facts reads nothing at execution.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
use std::num::NonZeroUsize;
use std::time::Duration;
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;
use worth_query_execution::facade::domain_computation::fact_verification_on_this_thread_for_test as fact_work;
use worth_query_execution::facade::domain_computation::installed_source_reads_on_this_thread_for_test as reads;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub struct ExecutionWork {
    pub foreign_runtime: bool,
    pub completed: bool,
    pub source_reads: u64,
    pub source_fact_admissions: u64,
}

/// Mints an admission bound to an observed source on `source`, then presents
/// it to `target`'s mutation execution and counts only that call's work.
pub fn execute_source_bound(
    source: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::OptionalSchema>,
    target: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::OptionalSchema>,
) -> ExecutionWork {
    type Schema = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::OptionalSchema;
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let external = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
        source.installed_schema(),
        "alice",
        Duration::from_secs(60),
        &request,
    );
    let input = Input {
        scope_key: crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned(),
        incomplete: false,
    };
    let (admission, principal_identity) = source
        .with_application_advancement(&request, |phase| {
            let selected = source
                .on_branch(source.current_world())
                .select(&phase)
                .unwrap();
            let principal_binding = source
                .installed_schema()
                .principal_binding(ConsumerPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &principal_binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = source
                .installed_schema()
                .installed_operation(OrdinaryProduce::reference())
                .unwrap();
            let mut admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &root,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let query = source
                .installed_schema()
                .installed_query_binding::<PlanarReadBinding<Schema>>()
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let plan = selected
                .admit_application_query(
                    &phase,
                    query.query(),
                    &access,
                    ApplicationQueryParameterSet::new(),
                    WorthQueryProductQueryControls::new(NonZeroUsize::new(1).unwrap(), &request),
                )
                .unwrap();
            let (_, observed) = source
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
                .into_admitted_disclosed()
                .into_output_demand_source()
                .into_single_source_for_test()
                .unwrap();
            source
                .bind_application_source_expectation::<OrdinaryBinding<Schema>, _>(
                    &phase,
                    &mut admission,
                    observed,
                    &input,
                )
                .unwrap()
                .consume_into(source.runtime(), &phase, &mut admission)
                .unwrap();
            (admission, *principal.principal_identity())
        })
        .unwrap();
    let identities =
        ApplicationMutationIdentities::<Schema, OrdinaryBinding<Schema>>::encode(&902, &input)
            .unwrap();
    target
        .with_application_advancement(&request, |phase| {
            let _observation =
                worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
            fact_work();
            let before = reads();
            let outcome = target.execute_mutation_handler::<OrdinaryBinding<Schema>>(
                &phase,
                &identities,
                &principal_identity,
                admission,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            );
            let source_reads = reads() - before;
            let source_fact_admissions = fact_work().source_admission_visits;
            let foreign_runtime = matches!(
                &outcome,
                Err(MutationHandlerExecutionDenial::Projection(denial))
                    if denial.kind() == WorthQueryOperationProjectionDenialKind::Authorization(
                        WorthQueryOperationAuthorizationDenialKind::Refused(
                            AuthorizationRefusal::ForeignRuntime,
                        ),
                    )
            );
            ExecutionWork {
                foreign_runtime,
                completed: matches!(outcome, Ok(HandlerResult::Completed(_))),
                source_reads,
                source_fact_admissions,
            }
        })
        .unwrap()
}

#[test]
fn a_foreign_admission_with_source_facts_is_refused_before_any_graph_read() {
    let source = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::OptionalSchema>(false);
    let target = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::OptionalSchema>(false);

    let control = execute_source_bound(&target, &target);
    assert!(control.completed, "the owned admission executes");
    assert!(
        control.source_fact_admissions > 0 && control.source_reads > 0,
        "the same door compares source facts and reads for an owned admission"
    );

    let foreign = execute_source_bound(&source, &target);
    assert!(foreign.foreign_runtime, "ownership is refused first");
    assert_eq!(foreign.source_fact_admissions, 0);
    assert_eq!(foreign.source_reads, 0);
}
