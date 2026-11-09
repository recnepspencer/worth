use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::ApplicationScalarValueBinding;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_world_with_label;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedAccountOmissionQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_omission_query::GovernedAccountOmissionResult;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosureDecisionFact;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosureOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosureReceiptPosture;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotResult;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryOmissionPosture;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQuerySourceExpectationDenialKind;

pub type GovernedResult =
    WorthQueryApplicationOneShotResult<GovernedAccountOmissionQuery, GovernedAccountOmissionResult>;

#[derive(Debug, Eq, PartialEq)]
pub struct ConsumerObservation {
    pub rows: Vec<GovernedAccountOmissionResult>,
    pub disclosure_posture: WorthQueryApplicationDisclosureReceiptPosture,
    pub classification: Option<String>,
    pub disclosed: Vec<AspectValue>,
    pub omitted: Vec<AspectValue>,
    pub decisions: Vec<WorthQueryApplicationDisclosureDecisionFact>,
    pub disclosure_decision_count: usize,
    pub capability_identity_present: bool,
    pub decision_identity_present: bool,
    pub decision_fact_count: usize,
    pub omission_posture: WorthQueryApplicationQueryOmissionPosture,
    pub result_count: usize,
    pub projected_field_count: usize,
    pub adjacency_list_read_count: usize,
    pub edge_scan_count: usize,
    pub observed_source_debug: Vec<String>,
}

#[test]
fn protected_label_difference_is_absent_from_every_one_shot_observable() {
    let left = observe("private-left");
    let right = observe("private-right");

    assert_eq!(left, right);
    assert_eq!(left.rows.len(), 1);
    assert_eq!(left.projected_field_count, 1);
    assert_eq!(left.adjacency_list_read_count, 0);
    assert_eq!(left.edge_scan_count, 0);
    assert_eq!(
        left.observed_source_debug,
        vec!["WorthQueryObservedSource { .. }"]
    );
    assert!(matches!(
        left.rows[0].status(),
        WorthQueryApplicationDisclosed::Disclosed(status) if status == "open"
    ));
    let WorthQueryApplicationDisclosed::Omitted(omission) = left.rows[0].label() else {
        panic!("the protected label must be omitted before application projection");
    };
    assert_eq!(omission.classification(), "account-omission");
    assert_eq!(
        omission.required_disclosure(),
        &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding::encode(
            &CapabilityDisclosure::PrivateLabel,
        )
        .unwrap()
    );
    assert!(matches!(
        left.rows[0].note(),
        WorthQueryApplicationDisclosed::Omitted(_)
    ));
    assert!(matches!(
        left.rows[0].activities(),
        WorthQueryApplicationDisclosed::Omitted(_)
    ));
    assert_eq!(left.disclosure_decision_count, 6);
    assert_eq!(
        left.decisions
            .iter()
            .filter(|decision| {
                decision.required_disclosure()
                    == &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding::encode(
                        &CapabilityDisclosure::PrivateLabel,
                    )
                    .unwrap()
            })
            .count(),
        5
    );
    assert_eq!(
        left.decisions
            .iter()
            .filter(|decision| {
                decision.outcome() == WorthQueryApplicationDisclosureOutcome::Omitted
            })
            .count(),
        5
    );
    let slots = left
        .decisions
        .iter()
        .map(|decision| decision.slot().clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(slots.len(), left.decisions.len());
    assert!(left
        .decisions
        .windows(2)
        .all(|decisions| decisions[0].slot() < decisions[1].slot()));
}

#[test]
fn omitted_dependency_cannot_be_used_as_a_complete_source_expectation() {
    let world = installed_capability_world_with_label("private");
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let result = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            execute_result(
                &(phase).test_host_request_for_test(),
                &host_custody.borrowed_request_for_test(),
                &world,
            )
        })
        .expect("the fixture read owns its installed host call");
    let source = result
        .observed_sources()
        .first()
        .expect("the selected row carries an opaque source observation");
    let denial = source
        .validate_completeness_for_test("GovernedAccountOmissionQuery")
        .expect_err("an omitted declared dependency must not authorize a mutation");
    assert_eq!(
        denial.kind(),
        WorthQuerySourceExpectationDenialKind::IncompleteFootprint
    );
}

pub fn observe(label: &str) -> ConsumerObservation {
    let world = installed_capability_world_with_label(label);
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    world
        .application
        .with_host_advancement(|phase| execute(&phase.test_host_request_for_test(), &world))
        .expect("the observation fixture owns its installed read")
}

pub fn execute(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &AuthorizationWorld,
) -> ConsumerObservation {
    capture_observation(&execute_result(
        phase_custody,
        &phase_custody.borrowed_request_for_test(),
        world,
    ))
}

pub fn execute_result(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
) -> GovernedResult {
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &phase_custody.phase_for_application_for_test(&world.application),
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application_for_test(&world.application),
            AccountIdentity::at("AccountIdentity"),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(GovernedAccountOmissionQuery::reference())
        .unwrap();
    let capability = admit_touch_account_capability(
        &(phase_custody).test_host_request_for_test(),
        world,
        &principal,
        &request,
    )
    .unwrap();
    let capability_session = capability.graph_work_session_identity_for_test();
    let capability_run = capability.graph_work_managed_run_identity_for_test();
    let capability_branch = capability.graph_work_branch().clone();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .admit_governed_application_query(
            &phase_custody.phase_for_application_for_test(&world.application),
            &query,
            &access,
            capability,
            ApplicationQueryParameterSet::<GovernedAccountOmissionQuery>::new(),
            worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                &request,
            ),
        )
        .unwrap();
    assert_ne!(plan.graph_work_session_identity(), capability_session);
    assert_ne!(plan.graph_work_managed_run_identity(), capability_run);
    assert_eq!(plan.graph_work_branch(), &capability_branch);
    assert!(plan.graph_work_capability_identity().is_some());
    assert!(plan.graph_work_decision_fact_count() >= 3);
    world
        .application
        .execute_application_query_one_shot(
            &phase_custody.phase_for_application_for_test(&world.application),
            plan,
        )
        .unwrap()
}

pub fn capture_observation(result: &GovernedResult) -> ConsumerObservation {
    let disclosure = result.receipt().disclosure();
    ConsumerObservation {
        rows: result.rows().to_vec(),
        disclosure_posture: disclosure.posture(),
        classification: disclosure.classification().map(str::to_owned),
        disclosed: disclosure.disclosed().to_vec(),
        omitted: disclosure.omitted().to_vec(),
        decisions: disclosure.decisions().to_vec(),
        disclosure_decision_count: disclosure.disclosure_decision_count(),
        capability_identity_present: disclosure.capability_authority_identity().is_some(),
        decision_identity_present: disclosure.decision_identity().is_some(),
        decision_fact_count: disclosure.authorization_decision_fact_count(),
        omission_posture: result.receipt().omission_posture(),
        result_count: result.receipt().result_count(),
        projected_field_count: result.receipt().projected_field_count(),
        adjacency_list_read_count: result.receipt().reads().adjacency_lists_read(),
        edge_scan_count: result.receipt().reads().adjacency_edges_inspected(),
        observed_source_debug: result
            .observed_sources()
            .iter()
            .map(|source| format!("{source:?}"))
            .collect(),
    }
}
