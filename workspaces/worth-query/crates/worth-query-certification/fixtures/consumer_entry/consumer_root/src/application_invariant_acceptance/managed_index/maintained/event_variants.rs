//! A distinct declared schema runs the ordered event Source beside its two
//! variants: the same membership under `SameAdmittedScope`, and one relation
//! in both ordered roles. Each loan lends the provider owner's exact delta.
#![allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "This variant composition instantiates the complete ordinary schema and provider; its court uses only its own operations and observations."
)]
#[path = "nested/authoring.rs"]
mod authoring;
#[path = "events/composition.rs"]
pub(super) mod composition;
#[path = "events/history.rs"]
mod history;
#[path = "../mutation.rs"]
mod mutation;
#[path = "events/paths.rs"]
mod paths;
#[path = "events/plain.rs"]
mod plain;
#[path = "../prefix.rs"]
pub(super) mod prefix;
#[path = "nested/query.rs"]
mod query;
#[path = "events/rule.rs"]
pub(super) mod rule;
#[path = "events/schema.rs"]
pub(super) mod schema;
#[path = "events/variants.rs"]
pub(super) mod variants;
/// This world installs the two variant Sources.
const VARIANT_SOURCES: bool = true;
#[path = "nested/shape.rs"]
mod shape;
use super::super::super::authentication;
use std::sync::Arc;
use worth_query_decl::facade::{application_query::*, application_schema::*};
use worth_query_host::facade::{application_entry::*, runtime::ExecutionAllocationPolicy};
const SCHEMA_OWNER: &str = "worth.query.consumer.ordered-event-variants";
const SOURCE_COLD_POLICY: ApplicationManagedIndexColdSource =
    ApplicationManagedIndexColdSource::AdmitRebuild;
const SOURCE_MAX_RESULTS: usize = 8192;
const PREFIX_MEMBERSHIP: ApplicationManagedIndexMembership =
    ApplicationManagedIndexMembership::CheckedSummarySourceTree;
const PREFIX_REFINEMENT: Option<ApplicationManagedIndexRefinement> = None;
const PREFIX_MAINTENANCE: ApplicationManagedIndexMaintenance =
    ApplicationManagedIndexMaintenance::WhenRequired;
const PREFIX_SOURCE_SCOPE: ApplicationManagedIndexSourceScope =
    ApplicationManagedIndexSourceScope::OrderedPredecessor;
fn source_path() -> ApplicationQueryRootPath<schema::Catalog, schema::Directory, schema::Item> {
    ApplicationQueryRootPath::from(schema::Directory::reference())
        .forward_derived(schema::LiveItems::reference())
}

fn source_graph_laws() -> ApplicationRelationGraphLaws {
    ApplicationRelationGraphLaws::unbounded()
}

/// An append beside an unchanged prefix row. The predecessor scope lends no
/// row in any index. The born scope is built new beside its sibling Sources
/// (#251), yet its ordered Source lends only its step relative to the
/// predecessor's membership, never the inherited x; the same-scope Source,
/// which has no predecessor, lends every member, its ordinary rows; the
/// Source whose one relation both introduces and retires lends nothing, and
/// its ordinary Source is empty, for retirement wins.
/// Which Sources an invocation lends from.
#[derive(Clone, Copy)]
enum Lend {
    None,
    Ordered,
    All,
}
/// One fresh world: `x` in the catalog, then B2 appended after it with `y`.
/// Returns the rule's records and the append's charged SDK work.
fn append_world(lend: Lend) -> (Arc<rule::Calls>, Rows, u64) {
    use std::sync::atomic::Ordering;
    let records = Arc::new(rule::Calls::default());
    records
        .skip_loan
        .store(matches!(lend, Lend::None), Ordering::SeqCst);
    records
        .only_ordered
        .store(matches!(lend, Lend::Ordered), Ordering::SeqCst);
    let world = super::host::install_event_variants(Arc::clone(&records));
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let item = request
        .advanced()
        .mutate(super::nested::edit::Edit::Item {
            directory: "catalog".into(),
            identity: "x".into(),
            name: "match-x".into(),
            root: true,
            incarnation: 1,
        })
        .idempotency(&5000u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(item.result().is_some(), "{item:?}");
    records.observed.lock().unwrap().clear();
    let (append, work) = super::court::prefix_growth::invoice(|| {
        request
            .advanced()
            .mutate(history::Edit::Append {
                rename: None,
                event: "B2".into(),
                previous: "catalog".into(),
                introduced: Some(("y".into(), "match-y".into())),
                retired: None,
                stray: None,
                adopt: None,
            })
            .without_source()
            .idempotency(&5001u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    assert!(append.result().is_some(), "{append:?}");
    let rows = |directory: &str| {
        let rows = request
            .query(query::ReadSource {
                directory: directory.into(),
            })
            .execute()
            .unwrap();
        rows.rows()
            .iter()
            .map(|row| row.coordinate)
            .collect::<std::collections::BTreeSet<_>>()
    };
    let (catalog, born) = (rows("catalog"), rows("B2"));
    let overlap = request
        .query(variants::ReadOverlap {
            directory: "B2".into(),
        })
        .execute()
        .unwrap();
    let overlap = overlap.rows().iter().map(|row| row.coordinate).collect();
    (records, [catalog, born, overlap], work)
}
type Rows = [std::collections::BTreeSet<(u32, u64, u32)>; 3];

/// An append beside an unchanged prefix row. The predecessor scope lends no
/// row in any index. The born scope is built new beside its sibling Sources,
/// so each of its live Sources lends every member, its ordinary rows; the
/// Source whose one relation both introduces and retires lends nothing, and
/// its ordinary Source is empty, for retirement wins.
#[test]
fn an_append_lends_no_unchanged_prefix_row_and_the_overlap_source_lends_nothing() {
    use std::collections::BTreeSet;
    let (records, [catalog, born, overlap], _) = append_world(Lend::All);
    assert_eq!(catalog.len(), 1, "the predecessor holds x");
    assert_eq!(born.len(), 2, "B2 holds x and y");
    assert!(overlap.is_empty(), "retirement wins in the overlap Source");
    let observed = records.observed.lock().unwrap();
    let born_scope = observed
        .iter()
        .find(|(index, _, [_, proposed])| *index == "ordered" && !proposed.is_empty())
        .expect("the born scope lends its step")
        .1;
    let mut seen = BTreeSet::new();
    for (index, at, [committed, proposed]) in observed.iter() {
        let proposed: BTreeSet<_> = proposed.iter().copied().collect();
        assert!(committed.is_empty(), "{index}: nothing left or changed");
        if *at != born_scope {
            assert!(proposed.is_empty(), "{index} lends at an unchanged scope");
            continue;
        }
        seen.insert(*index);
        let expected = match *index {
            "overlap" => BTreeSet::new(),
            "ordered" => born.difference(&catalog).copied().collect(),
            _ => born.clone(),
        };
        assert_eq!(proposed, expected, "{index}");
    }
    assert_eq!(seen, BTreeSet::from(["ordered", "same-scope", "overlap"]));
}

mod cost {
    use super::*;
    /// Each loan borrows its own prepared delta: lending from all three
    /// Sources costs near three times lending from one, never the square a
    /// per-loan scan of every staged Source would cost.
    #[test]
    fn each_sources_loan_is_constant_work_whatever_the_number_of_sources() {
        let (_, _, without) = append_world(Lend::None);
        let (_, _, one) = append_world(Lend::Ordered);
        let (_, _, all) = append_world(Lend::All);
        let (one, all) = (one.saturating_sub(without), all.saturating_sub(without));
        eprintln!("append loans: {one} for one Source, {all} for three");
        assert!(
            one > 0 && all <= one * 5,
            "loan SDK work: one {one}, three {all}"
        );
    }
}
