//! One checked collection uses the real graph-bound region producer and retained tree.
use super::*;
pub(in crate::checkpoint_recovery::computation_partition) mod lifecycle;
use worth_query_decl::facade::application_program::{
    ApplicationCollectionCompleteness, ApplicationCollectionContributor,
    ApplicationCollectionGrouping, ApplicationCollectionIncompletePosture,
    ApplicationCollectionLineage, ApplicationCollectionMeasures, ApplicationDerivedCollection,
};

pub(super) const CHECKED: u8 = 5;
const MISSING: u8 = 6;
pub(super) const MISBOUND: u8 = 7;
pub(super) const LIFECYCLE: u8 = 8;
pub(super) const CONFLICTING_READINESS: u8 = 9;
pub(super) const ORPHAN_OPERATION: u8 = 10;
pub(super) const ORPHAN_STAGE: u8 = 11;
pub(super) const fn declared(mode: u8) -> bool {
    matches!(
        mode,
        CHECKED
            | MISSING
            | MISBOUND
            | LIFECYCLE
            | CONFLICTING_READINESS
            | ORPHAN_OPERATION
            | ORPHAN_STAGE
    )
}

pub(super) struct Contributors;
impl ApplicationCollectionContributor for Contributors {
    const IDENTITY: &'static str = "neutral-region-entries";
}
pub(super) struct Groups;
impl ApplicationCollectionGrouping for Groups {
    const IDENTITY: &'static str = "neutral-region-key";
}
pub(super) struct Measures;
impl ApplicationCollectionMeasures for Measures {
    const IDENTITY: &'static str = "neutral-nonnegative-total";
}
pub(super) struct Lineage;
impl ApplicationCollectionLineage for Lineage {
    const IDENTITY: &'static str = "neutral-tracked-entry-facts";
}
pub(super) struct Complete;
impl ApplicationCollectionCompleteness for Complete {
    type Value = f64;
    type Stopped = u32;
    type Lineage = Lineage;
    const IDENTITY: &'static str = "neutral-nonnegative-total-completeness";
    fn check(value: &f64) -> Result<(), u32> {
        if value.is_finite() && *value >= 0.0 {
            Ok(())
        } else {
            Err(99)
        }
    }
}

macro_rules! collection {
    ($name:ident) => {
        pub(super) struct $name<const WORK: usize>;
        impl<const WORK: usize>
            ApplicationDerivedCollection<CheckpointSchema, PlanarFinalOutputFeature>
            for $name<WORK>
        {
            type Computation = OracleTotals<WORK>;
            type Completeness = Complete;
            type Contributor = Contributors;
            type Grouping = Groups;
            type Measures = Measures;
            type Lineage = Lineage;
            // Deliberately identical portable spelling: Rust binding identity still matters.
            const IDENTITY: &'static str = "neutral-region-collection";
            const APPLICABILITY: &'static str = "admitted-entry-set";
            const INCOMPLETE: ApplicationCollectionIncompletePosture =
                ApplicationCollectionIncompletePosture::RetainPending;
            const INCREMENTAL_UPDATE: &'static str = "native-retained-partitions";
        }
    };
}
collection!(RegionCollection);
collection!(MisboundCollection);

#[test]
fn collection_installation_refuses_missing_or_misbound_owner_before_initial_effects() {
    use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial as Denial;
    use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as Kind;
    fn refusal<const MODE: u8>() -> Kind {
        let reached = std::cell::Cell::new(false);
        let result =
            support::try_install_program_with_limits::<OracleProgram<false, TOTALS_WORK, 1, MODE>>(
                None,
                Default::default(),
                support::limits(
                    32,
                    support::invalidation(128 * 1024 * 1024, Some(1_000_000), 32),
                ),
                |graph| {
                    reached.set(true);
                    support::seed_cycle(graph);
                },
            );
        assert!(
            !reached.get(),
            "refusal precedes the actual initial-state callback"
        );
        match *result
            .err()
            .expect("unchecked collection installation refuses")
        {
            Denial::Contributions(denial) => denial.kind(),
            other => panic!("expected collection installation refusal, got {other:?}"),
        }
    }
    let _guard = checkpoint_recovery_test_guard();
    assert_eq!(refusal::<MISSING>(), Kind::MissingCollectionOwner);
    assert_eq!(refusal::<MISBOUND>(), Kind::CollectionOwnerMeaningMismatch);
}

#[test]
fn checked_collection_publishes_and_reuses_the_native_canonical_tree() {
    let _guard = checkpoint_recovery_test_guard();
    let application = installation::install_variant::<false, TOTALS_WORK, 1, CHECKED>(
        None,
        Default::default(),
        |graph| {
            facts::seed_set(graph, "even", -0.0);
            for (place, value) in [2.0, 3.0, 7.0].into_iter().enumerate() {
                seed_entry(
                    graph,
                    &["even"],
                    place,
                    super::super::execution::entry(place as u64, place as u32, value),
                );
            }
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (contacts, first) = demand(&request, &application);
    assert_eq!(contacts, 1);
    assert_eq!(first[0].outcome.as_ref().unwrap().0, 12.0_f64.to_bits());
    assert!(
        !first[0].published.is_empty(),
        "a genuine producer publication retains the tree"
    );
    assert_eq!(
        demand(&request, &application).0,
        0,
        "published collection is current"
    );
    edit(
        &request,
        &application,
        super::super::entry_edit::EntryEdit::new(
            "even",
            1,
            super::super::entry_edit::EntryFact::Value,
            5.0_f64.to_bits(),
        ),
        1,
    );
    let (_, next) = demand(&request, &application);
    assert_eq!(next[0].outcome.as_ref().unwrap().0, 14.0_f64.to_bits());
    assert_eq!(next[0].calls.gathers, 1);
    assert_eq!(next[0].calls.kernels, 1);
    assert_eq!(
        next[0].runs,
        [WorthQueryPartitionedComputationRunForTest::Incremental]
    );
    assert!(!next[0].published.is_empty());
}

#[test]
fn known_empty_collection_publishes_canonical_zero() {
    let _guard = checkpoint_recovery_test_guard();
    let application = installation::install_variant::<false, TOTALS_WORK, 1, CHECKED>(
        None,
        Default::default(),
        |graph| {
            facts::seed_set(graph, "even", -0.0);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (_, runs) = demand(&request, &application);
    assert_eq!(runs[0].outcome.as_ref().unwrap().0, (-0.0_f64).to_bits());
    assert!(!runs[0].published.is_empty());
    assert_eq!(demand(&request, &application).0, 0);
}

#[test]
fn declared_completeness_refusal_cannot_publish_the_computed_value() {
    let _guard = checkpoint_recovery_test_guard();
    let application = installation::install_variant::<false, TOTALS_WORK, 1, CHECKED>(
        None,
        Default::default(),
        |graph| {
            facts::seed_set(graph, "even", -0.0);
            seed_entry(
                graph,
                &["even"],
                0,
                super::super::execution::entry(0, 1, -1.0),
            );
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    room().clear();
    published_states();
    let mut handle = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram<false, TOTALS_WORK, 1, CHECKED>, RegionConnection>(
            &application,
        )
        .expect("the actual collection demand starts");
    let result = handle.advance(&request);
    assert!(
        result.is_err(),
        "completeness refusal stops native publication"
    );
    let runs = room();
    assert_eq!(runs.len(), 1);
    assert!(matches!(
        runs[0].outcome,
        Err(WorthQueryPartitionedComputationDenial::Owner(99))
    ));
    assert!(
        runs[0].calls.kernels > 0,
        "the real kernel ran before completeness refused"
    );
    assert!(
        published_states().is_empty(),
        "refused completion retained no published computation"
    );
}
