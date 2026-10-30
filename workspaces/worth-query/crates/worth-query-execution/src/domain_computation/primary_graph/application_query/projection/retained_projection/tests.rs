//! Existing World, real query completion and real Native field publications.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, AccountLabel, AccountSummaryResult,
    TestAccountSourceBinding,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
mod reads;
use reads::{read, set_label};

#[test]
fn original_projection_moves_disclosed_rows_and_shares_original_funding_until_final_close() {
    let world = installed_authorization_world(true);
    let query = world
        .application
        .installed_schema()
        .installed_query_binding::<TestAccountSourceBinding>()
        .unwrap()
        .into_query();
    let label_field = AccountLabel::reference();
    let label = query
        .read_graph()
        .projections()
        .iter()
        .find(|field| {
            (field.entity(), field.aspect(), field.field())
                == (
                    label_field.entity(),
                    label_field.aspect(),
                    label_field.field(),
                )
        })
        .unwrap();
    let registry = world.application.application_query_basis_observer();
    let baseline = registry.observe().active();
    let (observation, sibling, weak) = world.application.with_host_advancement(|phase| {
        let ordinary = read(&world, &phase, "open", false, true).unwrap();
        assert!(ordinary.result_set_observation().source().original_projection().is_none());
        let result = read(&world, &phase, "open", true, true).unwrap();
        assert_eq!(result.rows(), ordinary.rows(), "retention preserves ordinary decoding");
        let row = result.observed_sources()[0].retyped::<()>();
        let set = result.result_set_observation().source();
        let body = row.original_projection().unwrap();
        assert!(Arc::ptr_eq(body, set.original_projection().unwrap()));
        assert!(Arc::ptr_eq(body, row.clone().original_projection().unwrap()));
        assert_eq!(body.rows.len(), 1);
        assert_eq!(body.rows[0].entity_id(), row.model_root);
        assert_eq!(body.rows[0].field(label.slot_type()).unwrap().value(),
            &worth_foundational::facade::AspectValue::String("primary".into()));
        assert_eq!(body.disclosure.len(), query.read_graph().projections().len()
            + query.read_graph().relations().len());
        assert!(body.disclosure.iter().all(|(_, disclosed)| *disclosed));
        for (slot, _) in body.disclosure.iter() {
            assert!(query.read_graph().projections().iter()
                .map(|field| field.slot_key_identity())
                .chain(query.read_graph().relations().iter().map(|relation| relation.slot_key_identity()))
                .any(|original| Arc::ptr_eq(slot, &original)));
        }
        let body_quote = body.rows.iter().fold(
            (body.rows.capacity() * std::mem::size_of::<super::super::WorthQueryApplicationProjectionNode>()) as u64,
            |bytes, row| bytes + row.retained_bytes() as u64,
        ) + OriginalProjection::allocation_bytes();
        assert_eq!(body._memory.as_ref().unwrap().bytes(), body_quote);
        assert_eq!(body.disclosure.charged_payload_bytes(),
            Some((body.disclosure.len() * std::mem::size_of::<(
                Arc<worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey>, bool)>()) as u64));
        let original = Arc::clone(body);
        let weak = Arc::downgrade(body);
        let completed = result.into_admitted_disclosed().into_output_demand_source();
        let observation = completed.retain_inspection_observation().unwrap();
        let sibling = observation.clone();
        drop(completed);
        drop(row);
        set_label(&world, &phase, original.rows[0].entity_id(), label, "changed");
        let changed = read(&world, &phase, "open", true, true).unwrap();
        let changed_body = changed.result_set_observation().source().original_projection().unwrap();
        assert_eq!(original.rows[0].field(label.slot_type()).unwrap().value(),
            &worth_foundational::facade::AspectValue::String("primary".into()));
        assert_eq!(changed_body.rows[0].field(label.slot_type()).unwrap().value(),
            &worth_foundational::facade::AspectValue::String("changed".into()));
        set_label(&world, &phase, original.rows[0].entity_id(), label, "changed");
        let equal_rewrite = read(&world, &phase, "open", true, true).unwrap();
        let equal_body = equal_rewrite.result_set_observation().source().original_projection().unwrap();
        assert_eq!(changed_body.rows[0].field(label.slot_type()).unwrap().value(),
            equal_body.rows[0].field(label.slot_type()).unwrap().value(),
            "a genuine equal-value Native rewrite does not imply a changed cell");
        (observation, sibling, weak)
    }).unwrap();
    assert_eq!(registry.observe().active(), baseline);
    assert!(weak.upgrade().is_some());
    observation.close();
    assert!(weak.upgrade().is_some());
    sibling.close();
    assert!(
        weak.upgrade().is_none(),
        "last opaque clone releases original body and funding"
    );
    assert_eq!(registry.observe().active(), baseline);
}

#[test]
fn original_projection_retains_bound_empty_results_but_refuses_unbound_capture() {
    let world = installed_authorization_world(true);
    let registry = world.application.application_query_basis_observer();
    let baseline = registry.observe().active();
    let (observation, weak) = world
        .application
        .with_host_advancement(|phase| {
            let ordinary = read(&world, &phase, "no-match", false, true).unwrap();
            assert!(ordinary.rows().is_empty());
            assert!(ordinary
                .result_set_observation()
                .source()
                .original_projection()
                .is_none());
            let result = read(&world, &phase, "no-match", true, true).unwrap();
            assert!(result.rows().is_empty());
            assert!(result.observed_sources().is_empty());
            let body = result
                .result_set_observation()
                .source()
                .original_projection()
                .unwrap();
            assert!(body.rows.is_empty());
            assert!(
                !body.disclosure.is_empty(),
                "zero rows still retain every declared decision"
            );
            assert_eq!(
                body._memory.as_ref().unwrap().bytes(),
                OriginalProjection::allocation_bytes()
            );
            let weak = Arc::downgrade(body);
            let completed = result.into_admitted_disclosed().into_output_demand_source();
            let observation = completed.retain_inspection_observation().unwrap();
            let Err(error) = read(&world, &phase, "open", true, false) else {
                panic!("unbound capture must not publish a completed backing");
            };
            assert_eq!(
                error.kind(),
                WorthQueryApplicationOneShotDenialKind::ProjectionInspectionUnavailable
            );
            let unbound = read(&world, &phase, "open", false, false).unwrap();
            assert_eq!(unbound.rows().len(), 1);
            assert!(unbound
                .result_set_observation()
                .source()
                .original_projection()
                .is_none());
            (observation, weak)
        })
        .unwrap();
    assert_eq!(registry.observe().active(), baseline);
    assert!(weak.upgrade().is_some());
    observation.close();
    assert!(weak.upgrade().is_none());
    assert_eq!(registry.observe().active(), baseline);
}
