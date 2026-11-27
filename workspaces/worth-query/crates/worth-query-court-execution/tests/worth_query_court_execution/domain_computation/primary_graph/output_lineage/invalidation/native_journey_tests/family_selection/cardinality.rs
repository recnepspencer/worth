//! Courts 3 and 4 (public #429): a selection is not current once the family's
//! cardinality changes. The family spans the seed's retained-account output
//! and the generated computed-account output, so partition 7 of the seeded
//! account gaining or losing a head moves the selection between `Unique` and
//! `Ambiguous`.
use super::heads::retire;
use super::{certify, install_family, read_family, retained, Fact, FactMovement, Kind};
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_second_partition_head;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::own_write_computation::OwnWriteOutputs;

/// The family over both outputs, installed after the seed so that a read
/// can record `Unique` before partition 7 gains its head.
fn install_both(world: &AuthorizationWorld) {
    install_family(world, vec![retained(), (EqualityTypeId::of::<OwnWriteOutputs>(), "computed-account".to_owned())]);
}

/// The one `FamilySelection` a read recorded.
fn selection(facts: &[Fact]) -> &Fact {
    let [selection @ Fact::FamilySelection { .. }] = facts else {
        panic!("the read must record exactly one FamilySelection fact: {facts:?}");
    };
    selection
}

/// Court 3: a `Unique` read is not current once a second partition of the
/// same source gains a current head.
#[test]
fn unique_to_ambiguous_is_not_current() {
    with_second_partition_head(
        |world| {
            install_both(world);
            let (kind, facts) = read_family(world, "1");
            assert_eq!(kind, Kind::Unique, "only the seed's head is current before partition 7 completes");
            facts
        },
        |world, _receipt, facts| {
            let (fresh, _) = read_family(world, "1");
            assert_eq!(fresh, Kind::Ambiguous, "a fresh read selects both partitions' heads");
            assert_eq!(
                certify(world, selection(&facts)),
                FactMovement::from_equal(false),
                "a Unique selection is not current once the family is ambiguous"
            );
        },
    );
}

/// Court 4: an `Ambiguous` read is not current once one of its two heads
/// retires.
#[test]
fn ambiguous_to_unique_is_not_current() {
    with_second_partition_head(install_both, |world, _receipt, ()| {
        let (kind, facts) = read_family(world, "1");
        assert_eq!(kind, Kind::Ambiguous, "both partitions' heads are current");
        retire(world, "1", "generated");
        let (fresh, _) = read_family(world, "1");
        assert_eq!(fresh, Kind::Unique, "a fresh read selects the seed's head alone");
        assert_eq!(
            certify(world, selection(&facts)),
            FactMovement::from_equal(false),
            "an Ambiguous selection is not current once one head retired"
        );
    });
}
