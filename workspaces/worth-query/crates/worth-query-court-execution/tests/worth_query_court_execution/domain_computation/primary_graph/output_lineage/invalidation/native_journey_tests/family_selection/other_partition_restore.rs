//! Court 14 (d) and (e), public #429 blocker 1: a reader recorded
//! `Unique(E1)` from the seed's witnessed head, then a second partition gains
//! an unwitnessed restore head `H2` for the generated account `E2` through
//! the lineage's restoration door. Re-resolution gives `H2` the read's own
//! verdict, so a current `H2` moves the reader once and a stale or uncovered
//! one never does. Each case also runs through the site 2 rebase door with
//! an applied overlay that supplies `E1` and masks the seed's partition.
use super::heads::{account, certify_rebased, overlaid, Producer};
use super::{certify, install_family, read_family, retained, Fact, FactMovement, Kind};
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::{
    TypeId, WorthQueryApplicationObservedFact,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_second_partition_head;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionOutputs;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

/// `H2`'s partition.
const PARTITION: [u8; 32] = [0x72; 32];

/// The one `FamilySelection` a read recorded.
fn selection(facts: &[Fact]) -> Fact {
    let [selection @ Fact::FamilySelection { .. }] = facts else {
        panic!("the read must record exactly one FamilySelection fact: {facts:?}");
    };
    selection.clone()
}

/// Run every variant, then fail naming each one that failed, so a court
/// reports all of its variants rather than stopping at the first.
fn every_variant<V: std::fmt::Debug>(
    variants: impl IntoIterator<Item = V>,
    mut run: impl FnMut(&V),
) {
    let failed: Vec<String> = variants
        .into_iter()
        .filter_map(|variant| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&variant)))
                .err()
                .map(|_| format!("{variant:?}"))
        })
        .collect();
    assert!(failed.is_empty(), "variants failed: {failed:?}");
}

/// How `H2` stands at certification.
#[derive(Clone, Copy, Debug)]
enum Restore {
    /// Its checkpoint facts are current and cover the output.
    Current,
    /// Its checkpoint facts are from another revision of `E2`'s output.
    Stale,
    /// Its checkpoint facts do not cover the output.
    Uncovered,
}

/// What `test` sees after `H2` is recorded: the producer, `H2`, and the
/// reader's `Unique(E1)` selection, overlaid as its own commit's when
/// `rebased`, with the certification door that matches.
struct Partitioned<'w> {
    producer: Producer<'w>,
    head: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::restoration::RestoredRecord,
    selection: Fact,
    rebased: bool,
}

impl Partitioned<'_> {
    fn world(&self) -> &AuthorizationWorld {
        self.producer.world
    }

    /// The first certification: the rebase door for an overlaid selection.
    fn certify_first(&self) -> FactMovement {
        if self.rebased {
            certify_rebased(self.world(), &self.selection)
        } else {
            certify(self.world(), &self.selection)
        }
    }
}

fn unique_reader_then_restore(
    restore: Restore,
    rebased: bool,
    test: impl FnOnce(&Partitioned<'_>),
) {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    with_second_partition_head(
        |world| {
            install_family(world, vec![retained()]);
            let (kind, facts) = read_family(world, "1");
            assert_eq!(
                kind,
                Kind::Unique,
                "the seed's head is the family's one member"
            );
            selection(&facts)
        },
        |world, _receipt, recorded| {
            let producer = Producer::of(world, "1");
            let generated = account(world, "generated");
            let facts = match restore {
                Restore::Current => producer.checkpoint_facts(generated),
                Restore::Stale => producer.stale_checkpoint_facts(generated),
                Restore::Uncovered => vec![WorthQueryApplicationObservedFact::SourceEntity {
                    entity_id: producer.entity,
                }],
            };
            let head = producer.record(
                TypeId::of_for_test::<RetentionOutputs>(),
                PARTITION,
                generated,
                facts,
                false,
            );
            let selection = if rebased {
                overlaid(
                    &recorded,
                    &[producer.entity],
                    &producer.partitions_except(PARTITION),
                )
            } else {
                recorded
            };
            test(&Partitioned {
                producer,
                head,
                selection,
                rebased,
            });
        },
    );
}

/// (d) A current `H2` is a second member. Certifying answers `Moved` without
/// retaining `H2`'s witness; the one re-execution reads `Ambiguous([E1, E2])`
/// as a fresh `current_output` read would, and its selection certifies
/// `Unmoved`.
#[test]
fn a_current_restore_in_another_partition_moves_a_unique_reader_once() {
    every_variant([false, true], |&rebased| {
        unique_reader_then_restore(Restore::Current, rebased, |partitioned| {
            assert_eq!(
                partitioned.certify_first(),
                FactMovement::from_equal(false),
                "a current restore root in another partition is a member the reader did not select (rebased {rebased})"
            );
            assert!(
                partitioned.producer.unwitnessed(&partitioned.head),
                "certification never retains H2's witness (rebased {rebased})"
            );
            let (fresh, facts) = read_family(partitioned.world(), "1");
            assert_eq!(
                fresh,
                Kind::Ambiguous,
                "the one re-execution reads both partitions' heads (rebased {rebased})"
            );
            assert_eq!(
                certify(partitioned.world(), &selection(&facts)),
                FactMovement::from_equal(true),
                "the re-execution's selection certifies (rebased {rebased})"
            );
        });
    });
}

/// (e) A stale or uncovered `H2` is no member, as at read time, so
/// certification converges: at most one re-execution, then `Unmoved`.
#[test]
fn a_stale_or_uncovered_restore_in_another_partition_converges() {
    let variants = [Restore::Stale, Restore::Uncovered]
        .into_iter()
        .flat_map(|restore| [(restore, false), (restore, true)]);
    every_variant(variants, |&(restore, rebased)| {
        unique_reader_then_restore(restore, rebased, |partitioned| {
            let mut movement = partitioned.certify_first();
            let mut executions = 0;
            while movement != FactMovement::from_equal(true) {
                executions += 1;
                assert!(
                        executions <= 1,
                        "a {restore:?} restore root converges in at most one re-execution (rebased {rebased})"
                    );
                let (fresh, facts) = read_family(partitioned.world(), "1");
                assert_eq!(
                    fresh,
                    Kind::Unique,
                    "a {restore:?} restore root is no member (rebased {rebased})"
                );
                movement = certify(partitioned.world(), &selection(&facts));
            }
        });
    });
}
