use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::time::{Duration, UNIX_EPOCH};

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::ApplicationScalarValueBinding;

use super::super::super::fixture::{
    admit_touch_account_capability, installed_capability_world_with_label, live_scope,
    AccountIdentity, AuthorizationWorld, CapabilityDisclosure, GovernedAccountOmissionQuery,
    GovernedAccountOmissionResult,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDisclosed, WorthQueryApplicationDisclosureDecisionFact,
    WorthQueryApplicationDisclosureOutcome, WorthQueryApplicationDisclosureReceiptPosture,
    WorthQueryApplicationOneShotResult, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryOmissionPosture, WorthQueryPrincipalResolutionMode,
};

type GovernedResult =
    WorthQueryApplicationOneShotResult<GovernedAccountOmissionQuery, GovernedAccountOmissionResult>;

#[derive(Debug, Eq, PartialEq)]
struct ConsumerObservation {
    rows: Vec<GovernedAccountOmissionResult>,
    disclosure_posture: WorthQueryApplicationDisclosureReceiptPosture,
    classification: Option<String>,
    disclosed: Vec<AspectValue>,
    omitted: Vec<AspectValue>,
    decisions: Vec<WorthQueryApplicationDisclosureDecisionFact>,
    disclosure_decision_count: usize,
    capability_identity_present: bool,
    decision_identity_present: bool,
    decision_fact_count: usize,
    omission_posture: WorthQueryApplicationQueryOmissionPosture,
    result_count: usize,
    projected_field_count: usize,
    adjacency_list_read_count: usize,
    edge_scan_count: usize,
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
        &super::super::super::fixture::CapabilityDisclosureBinding::encode(
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
                    == &super::super::super::fixture::CapabilityDisclosureBinding::encode(
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

fn observe(label: &str) -> ConsumerObservation {
    let world = installed_capability_world_with_label(label);
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    execute(&world)
}

fn execute(world: &AuthorizationWorld) -> ConsumerObservation {
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(GovernedAccountOmissionQuery::reference())
        .unwrap();
    let capability = admit_touch_account_capability(world, &principal, &request).unwrap();
    let capability_session = capability.graph_work_session_identity();
    let capability_run = capability.graph_work_managed_run_identity();
    let capability_branch = capability.graph_work_branch().clone();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_governed_application_query(
            &query,
            &access,
            capability,
            ApplicationQueryParameterSet::<GovernedAccountOmissionQuery>::new(),
            crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(256).unwrap(),
                &request,
            ),
        )
        .unwrap();
    assert_ne!(plan.graph_work_session_identity(), capability_session);
    assert_ne!(plan.graph_work_managed_run_identity(), capability_run);
    assert_eq!(plan.graph_work_branch(), &capability_branch);
    assert!(plan.graph_work_capability_identity().is_some());
    assert!(plan.graph_work_decision_fact_count() >= 3);
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    capture_observation(&result)
}

fn capture_observation(result: &GovernedResult) -> ConsumerObservation {
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
        adjacency_list_read_count: result.receipt().adjacency_list_read_count(),
        edge_scan_count: result.receipt().edge_scan_count(),
    }
}
