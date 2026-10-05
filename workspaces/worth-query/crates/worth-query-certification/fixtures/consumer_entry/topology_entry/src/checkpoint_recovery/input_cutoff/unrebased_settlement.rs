//! A settlement its commit did not seal as exact is verified in full.

use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};

/// A body whose profile kind only the alternate producer serves, and that
/// producer declares the Initial posture alone.
const ROOT: &str = "manual-a";

/// The row has a sealed witness and a registered settlement, so the cutoff
/// verifies it in full and reuses it. Selection that sent it to the Preserve
/// posture would refuse every later demand: this kind has no Preserve
/// producer.
#[test]
fn a_settlement_verified_in_full_is_reused_where_no_producer_preserves() {
    let _guard = checkpoint_recovery_test_guard();
    let source_work =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard()
            .limits()
            .source_currentness_work();
    let application = support::install_program_with_seed::<CheckpointProgram>(
        None,
        Default::default(),
        32,
        128 * 1_024 * 1_024,
        u64::try_from(source_work).unwrap(),
        seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);

    application.leave_next_producer_settlement_unsealed_for_test();
    let performed = direct_demand(&request, &application, ROOT);
    assert_eq!(performed.producer_contacts_in_this_demand(), 1);
    drop(performed);

    for _ in 0..2 {
        let before = request.retain_read().unwrap();
        let reused = direct_demand(&request, &application, ROOT);
        assert_eq!(reused.producer_contacts_in_this_demand(), 0);
        drop(reused);
        let after = request.retain_read().unwrap();
        assert_eq!(before.selected_commit(), after.selected_commit());
    }
}

fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>) {
    let key = |name: &str| {
        WorthQueryApplicationEntityKey::<CheckpointSchema, Body>::new(format!("manual-{name}"))
            .unwrap()
    };
    for (name, x, y) in [("a", 1, 1), ("b", 10, 1), ("c", 1, 10)] {
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Body::reference::<CheckpointSchema>(),
                    key(name),
                )
                .field(
                    BodyKey::reference::<CheckpointSchema>(),
                    format!("manual-{name}"),
                )
                .field(Length::reference::<CheckpointSchema>(), length(y + 1))
                .field(PositionX::reference::<CheckpointSchema>(), length(x))
                .field(PositionY::reference::<CheckpointSchema>(), length(y)),
            )
            .unwrap();
    }
    for (from, to) in [("a", "b"), ("b", "c"), ("c", "a")] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarSuccessor::reference::<CheckpointSchema>(),
                format!("manual-{from}-to-{to}"),
                key(from),
                key(to),
            ))
            .unwrap();
    }
}
