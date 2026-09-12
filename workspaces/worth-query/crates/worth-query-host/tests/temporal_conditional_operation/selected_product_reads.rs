use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, primary_graph,
};

use super::product_query_support::{
    assert_security_work, controls, fork_relational_product, principal, product_identity,
};
use super::{schema::*, world};

#[test]
fn selected_data_stays_pinned_while_fresh_security_and_sibling_reads_progress() {
    let world = world::CourtroomWorld::publish("ready");
    let request = world::request_scope();
    let principal = principal(&world, &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let old = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    let old_commit = old.product().selected_commit().clone();
    let sibling = fork_relational_product(&world, &old, "read-sibling", "read-sibling-data");
    let selected = old;
    let scope = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_string(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = selected
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            controls(&request),
        )
        .unwrap();

    world.change_input_after_query_admission("changed-after-admission");
    let current = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    assert_ne!(current.product().selected_commit(), &old_commit);
    let current_branch = current.product().product_branch();
    drop(current);
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert_eq!(result.rows()[0].input, "payload");
    assert_eq!(
        product_identity(result.receipt()).selected_commit(),
        &old_commit
    );
    assert_security_work(result.receipt());

    for (product, expected) in [
        (sibling, "payload"),
        (current_branch, "changed-after-admission"),
    ] {
        let selected = world.application.on_branch(product).select().unwrap();
        let scope = selected
            .resolve_entity(
                IntentIdentityField::reference(),
                "intent-1".to_string(),
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let access =
            primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let plan = selected
            .admit_application_query(
                &query,
                &access,
                ApplicationQueryParameterSet::new(),
                controls(&request),
            )
            .unwrap();
        let result = world
            .application
            .execute_application_query_one_shot(plan)
            .unwrap();
        assert_eq!(result.rows()[0].input, expected);
        assert_eq!(product_identity(result.receipt()).product_branch(), product);
        assert_security_work(result.receipt());
    }
    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
}

#[test]
fn revoke_after_product_query_admission_denies_a_and_leaves_b_security_independent() {
    let world = world::CourtroomWorld::publish("ready");
    let request = world::request_scope();
    let principal = principal(&world, &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let source = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    let sibling =
        fork_relational_product(&world, &source, "security-sibling", "security-sibling-data");
    let selected_a = source;
    let scope_a = selected_a
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_string(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access_a =
        primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope_a);
    let plan_a = selected_a
        .admit_application_query(
            &query,
            &access_a,
            ApplicationQueryParameterSet::new(),
            controls(&request),
        )
        .unwrap();
    let selected_b = world.application.on_branch(sibling).select().unwrap();
    let scope_b = selected_b
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_string(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access_b =
        primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope_b);
    let plan_b = selected_b
        .admit_application_query(
            &query,
            &access_b,
            ApplicationQueryParameterSet::new(),
            controls(&request),
        )
        .unwrap();

    world.revoke_principal_on_default_product();
    let denied = world
        .application
        .execute_application_query_one_shot(plan_a)
        .err()
        .expect("fresh A security must observe revocation");
    assert_eq!(
        denied.kind(),
        primary_graph::WorthQueryApplicationOneShotDenialKind::StalePrincipal
    );
    let result_b = world
        .application
        .execute_application_query_one_shot(plan_b)
        .unwrap();
    assert_eq!(result_b.rows()[0].input, "payload");
    assert_eq!(
        product_identity(result_b.receipt()).product_branch(),
        sibling
    );
    assert_security_work(result_b.receipt());

    // Both entry and execution must resolve B security, even after MAIN revoked it.
    let selected_b = world.application.on_branch(sibling).select().unwrap();
    let fresh_b = selected_b
        .admit_application_query(
            &query,
            &access_b,
            ApplicationQueryParameterSet::new(),
            controls(&request),
        )
        .unwrap();
    let fresh_b = world
        .application
        .execute_application_query_one_shot(fresh_b)
        .unwrap();
    assert_eq!(fresh_b.rows()[0].input, "payload");
    assert_security_work(fresh_b.receipt());
    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
}
