//! A host default switch cannot replace the descriptor's issued branch.
use super::*;
use crate::application_program::ConsumerFixedAction;
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationAdvanceDenial,
    WorthQueryOperationCloseDenial, WorthQueryOperationDeliveryDenial,
    WorthQueryOperationProgress as Progress,
};
use worth_query_host::facade::primary_graph::{
    query_read_kernel_entries_on_this_thread_for_test as query_contacts,
    WorthQueryExecutionObservation,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};

#[test]
fn fixed_branch_affinity_default_switch_preserves_original_owner() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let world = installation::install(&crate::installed_schema());
    let runtime = world.application.runtime();
    let runtime_root = runtime.current_world();
    let original_branch = runtime
        .with_host_advancement(|phase| {
            runtime
                .branches()
                .fork(runtime_root)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let other_branch = runtime_root;
    assert_ne!(original_branch, runtime_root);
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let mut host_default = original_branch;
    let request = world
        .application
        .request(&principal, &scope)
        .on_branch(host_default);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let operation = request
        .mutate(input.clone())
        .expect_source(source)
        .idempotency_owned(10_191)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, denial)| panic!("construction: {denial:?}"));
    assert_eq!(operation.branch(), original_branch);
    host_default = other_branch;
    let wrong = world
        .application
        .request(&principal, &scope)
        .on_branch(host_default);
    assert_eq!(
        runtime.current_world(),
        runtime_root,
        "runtime root is immutable; the host selected another default"
    );
    let before = runtime
        .with_host_advancement(|phase| wrong.retain_read(&phase).unwrap())
        .unwrap();
    let before_data = wrong
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec();
    let before_queries = query_contacts();
    let operation = match operation.advance(&wrong) {
        Advance::Refused {
            operation,
            denial: WorthQueryOperationAdvanceDenial::FreshRequestMismatch,
        } => operation,
        _ => panic!("foreign branch returns the initial owner before effects"),
    };
    assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
    assert_eq!(query_contacts(), before_queries);

    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let fresh = world
        .application
        .request(&fresh_principal, &fresh_scope)
        .on_branch(operation.branch());
    let operation = match operation.advance(&fresh) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("returned owner reaches genuine source publication"),
    };
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let before_queries = query_contacts();
    let operation = match operation.advance(&wrong) {
        Advance::Refused {
            operation,
            denial: WorthQueryOperationAdvanceDenial::FreshRequestMismatch,
        } => operation,
        _ => panic!("foreign continuation preserves performed native custody"),
    };
    assert_eq!(operation.branch(), original_branch);
    assert_eq!(query_contacts(), before_queries);
    let mut operation = Some(operation);
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&fresh) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            _ => panic!("original branch native output progression"),
        })
        .expect("real fixed graph settles");
    assert_eq!(settled.branch(), original_branch);
    assert_eq!(settled.settlement().output_count(), 4);
    let producer_contacts = settled.settlement().root_producer_contacts_in_this_demand();
    let before_queries = query_contacts();
    assert!(matches!(
        settled.read_current(&wrong),
        Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch)
    ));
    assert_eq!(
        query_contacts(),
        before_queries,
        "foreign delivery enters no query kernel"
    );
    assert_eq!(
        settled.settlement().root_producer_contacts_in_this_demand(),
        producer_contacts
    );
    let (settled, denial) = settled
        .close(&wrong)
        .err()
        .expect("foreign close returns settled custody");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::FreshRequestMismatch
    ));
    let delivery_request = world
        .application
        .request(&fresh_principal, &fresh_scope)
        .on_branch(settled.branch());
    let delivery = settled.read_current(&delivery_request).unwrap();
    assert!(
        query_contacts() > before_queries,
        "the valid delivery twin enters the real query kernel"
    );
    assert_eq!(delivery.rows()[0].value, length(3));
    assert_eq!(producer_contacts, 1);
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let after = runtime
        .with_host_advancement(|phase| wrong.retain_read(&phase).unwrap())
        .unwrap();
    assert_eq!(after.selected_commit(), before.selected_commit());
    assert_eq!(
        wrong
            .query(PlanarRead {
                body_key: input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows(),
        before_data.as_slice()
    );
    settled
        .close(&delivery_request)
        .unwrap_or_else(|(_, denial)| panic!("fresh original branch close: {denial:?}"));
}
