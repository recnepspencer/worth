//! Original consumed lineage is checked before any source-triggered regeneration.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryOutputCurrentnessDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind as Kind;

fn assert_superseded(answer: Result<(), WorthQueryOutputCurrentnessDenial>, subject: &str) {
    assert!(
        matches!(&answer, Err(WorthQueryOutputCurrentnessDenial::Output(stop))
            if stop.kind() == Kind::Superseded),
        "{subject}: expected the original lineage to be superseded, got {answer:?}"
    );
}

/// The exact native output cells, also the ChainFamily's direct source query.
macro_rules! output_values {
    ($request:expr) => {
        ["anchor-a", "anchor-b", "anchor-c", "anchor-source-b"].map(|key| {
            let read = $request
                .query(PlanarOutputRead {
                    body_key: key.into(),
                })
                .execute()
                .unwrap();
            read.rows().to_vec()
        })
    };
}

#[test]
fn retained_consumer_currentness_checks_original_upstream_before_regeneration() {
    let _guard = checkpoint_recovery_test_guard();
    binding::take_all_decisions();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    let a_original = settled_in_one_advance!(a, request, "the settled root");
    let b_original = settled_in_one_advance!(b, request, "the settled consumer");
    let c_original = settled_in_one_advance!(c, request, "the settled descendant");
    let d_original = settled_in_one_advance!(d, request, "the settled sibling");
    // The retained image is after every original publication, not before B/C.
    let older = request.retain_current_read().unwrap();
    let original_values = output_values!(request.at(&older));
    let original_decisions = binding::decisions_snapshot();
    assert_eq!(original_values[0][0].value, length(2));

    // This authentic performed source edit does not advance an output demand.
    change_root_input!(request, application, 5, 0x9176_7f20_u64);
    let fresh = request.retain_current_read().unwrap();
    let selected = request.at(&fresh);
    let source = selected
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows()[0].y, length(5));
    drop(source);
    // A's own source fact moved at this same image while its output stayed old.
    // This is a semantic currentness check, not an assertion about cache marks.
    assert_superseded(
        selected.require_current_output_demand(&a_original, None),
        "the original root's direct source fact moved",
    );
    let consumer = selected.require_current_output_demand(&b_original, None);
    let descendant = selected.require_current_output_demand(&c_original, None);
    selected
        .require_current_output_demand(&d_original, None)
        .unwrap();
    request
        .at(&older)
        .require_current_output_demand(&b_original, None)
        .unwrap();
    request
        .at(&older)
        .require_current_output_demand(&c_original, None)
        .unwrap();
    assert_eq!(
        output_values!(selected),
        original_values,
        "A's artifact and B/C's actual direct source projections remain unchanged"
    );
    assert_eq!(
        binding::decisions_snapshot(),
        original_decisions,
        "checking currentness cannot replay B/C's producer decisions"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        fresh.selected_commit(),
        "checking currentness cannot publish another output"
    );
    // Pre-fix intended failure: all preceding semantic/unchanged/historical
    // assertions pass, but the omitted consumed edge incorrectly returns Ok.
    assert_superseded(consumer, "the original consumer before regeneration");
    assert!(
        matches!(&descendant, Err(WorthQueryOutputCurrentnessDenial::Output(stop))
            if stop.kind() == Kind::SchedulingDeferred),
        "the deeper original edge is genuinely pending, not current: {descendant:?}"
    );

    // Only now may the original native interests drive actual regeneration.
    settled_in_one_advance!(
        d,
        request,
        "the original interests refresh their dirty closure"
    );
    let renewed = settled_in_one_advance!(b, request, "the regenerated consumer");
    let now = request.retain_current_read().unwrap();
    request
        .at(&now)
        .require_current_output_demand(&renewed, None)
        .unwrap();
    assert_eq!(output_values!(request.at(&now))[0][0].value, length(6));
    assert_eq!(
        binding::decisions_snapshot()
            .iter()
            .filter(|(key, _)| key == "anchor-b")
            .map(|(_, inputs)| inputs.clone())
            .collect::<Vec<_>>(),
        [vec![2], vec![6]],
        "one genuine new B decision consumes the changed A output"
    );
    a.close();
    b.close();
    c.close();
    d.close();
    assert!(matches!(
        b.advance(&request),
        Err(WorthQueryApplicationOutputDemandDenial::Closed)
    ));
}
