//! Governance matches only the managed session that minted it.
use super::*;

#[test]
fn governance_matches_only_the_managed_session_that_minted_it() {
    let context = governance_context("primary");
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    let foreign = governance_context("foreign");
    let foreign_query = foreign
        .world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    let foreign_access =
        WorthQueryApplicationQueryAccessContext::new(&foreign.principal, &foreign.account);
    let foreign_plan = foreign
        .world
        .application
        .with_host_advancement(|phase| {
            admit_plan(&phase, &foreign, &foreign_query, &foreign_access)
        })
        .expect("the foreign fixture owns its admission call");
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();

            let first = admit_plan(
                &host_custody.phase_for_application_for_test(&context.world.application),
                &context,
                &query,
                &access,
            );
            let second = admit_plan(
                &host_custody.phase_for_application_for_test(&context.world.application),
                &context,
                &query,
                &access,
            );

            assert!(governance_matches_plan(&first, &first));
            assert!(!governance_matches_plan(&first, &second));
            assert_ne!(
                first.graph_work_session_identity(),
                second.graph_work_session_identity()
            );
            assert_ne!(
                first.graph_work_managed_run_identity(),
                second.graph_work_managed_run_identity()
            );

            assert!(
                !(*first.governance_for_test()).computation_matches_for_test(
                    &foreign_plan.graph_work,
                    (*foreign_plan.runtime_authority_for_test()),
                    (*first.query_for_test()).identity(),
                    (*first.parameters_for_test()).identity(),
                    first.principal.principal_entity_id_for_test(),
                    (*first.scope_for_test()).entity_id_for_test(),
                )
            );
        })
        .expect("the fixture block owns its installed host call")
}
