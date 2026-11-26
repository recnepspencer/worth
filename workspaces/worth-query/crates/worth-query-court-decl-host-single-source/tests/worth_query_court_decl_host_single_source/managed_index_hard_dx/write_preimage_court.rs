//! W8B-3: `UpdateAssetHandler` writes thirteen declared fields with no
//! discarded read, because `write_target` observed their pre-images. Those
//! pre-images still guard the commit: a competitor that commits a write to the
//! same row between the decision and the commit makes the original Stale
//! instead of being overwritten, while a competitor on another row does not.
use std::cell::Cell;
use std::rc::Rc;

use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

use super::harness::{install, update_asset};
use super::operations::{race_next_update_on_this_thread, AssetSpec, UpdateAsset};
use super::schema::*;
use super::Host;

fn asset(key: u64, title: &str) -> AssetSpec {
    AssetSpec::i64_only(key, title, (10, 30), [0, 0, 0], [5, 5, 5])
}

fn title_of(host: &Host, key: u64) -> String {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let query = FindByTitlePrefix {
        floor: "floor-1".into(),
        prefix: String::new(),
    };
    request
        .query(query.clone())
        .ensure_index_ready::<TitlePrefixIndex>()
        .unwrap();
    let published = request.query(query).execute().unwrap();
    published
        .rows()
        .iter()
        .find(|row| row.key == key)
        .map(|row| row.title.clone())
        .expect("the asset is listed")
}

/// Runs one update on this thread and returns its raw outcome.
fn try_update(
    host: &Host,
    spec: AssetSpec,
    idem: u64,
) -> WorthQueryApplicationMutationOutcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(UpdateAsset {
            key: spec.key,
            spec,
        })
        .without_source()
        .idempotency(&idem)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

/// Arms a competitor that commits `spec` from another thread while the next
/// update on this thread sits between its decision and its commit. The cell
/// receives the Home's commit-pulse version right after the competitor.
fn race_with(host: &Host, spec: AssetSpec, idem: u64) -> Rc<Cell<Option<u64>>> {
    let competitor_host = host.clone();
    let after_competitor = Rc::new(Cell::new(None));
    let recorded = Rc::clone(&after_competitor);
    race_next_update_on_this_thread(move || {
        std::thread::scope(|threads| {
            threads
                .spawn(|| update_asset(&competitor_host, spec, idem))
                .join()
                .expect("the competitor commits");
        });
        recorded.set(Some(competitor_host.runtime().commit_pulse().version()));
    });
    after_competitor
}

#[test]
fn an_update_without_discarded_reads_commits_and_a_racing_write_to_its_row_goes_stale() {
    let host = install(&[asset(1, "alpha"), asset(2, "beta")], None, None);

    // Every written field's pre-image came from `write_target`; the commit
    // advances the in-memory Home's commit pulse.
    let pulse = host.runtime().commit_pulse();
    let before = pulse.version();
    update_asset(&host, asset(1, "alpha-2"), 10);
    assert_eq!(title_of(&host, 1), "alpha-2");
    assert!(pulse.version() > before);

    // A competitor on another row leaves the original's pre-images intact.
    drop(race_with(&host, asset(2, "beta-2"), 11));
    let unrelated = try_update(&host, asset(1, "alpha-3"), 12);
    assert!(
        matches!(
            unrelated,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{unrelated:?}"
    );
    assert_eq!(
        (title_of(&host, 1), title_of(&host, 2)),
        ("alpha-3".to_owned(), "beta-2".to_owned())
    );

    // A competitor on the same row changes exactly one observed pre-image (the
    // title; every other field keeps its value): the original goes Stale on
    // that one fact, commits nothing, and the competitor's write stands.
    let after_competitor = race_with(&host, asset(1, "competitor"), 13);
    let racing = try_update(&host, asset(1, "original"), 14);
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Stale(
        stale,
    )) = &racing
    else {
        panic!("a write racing a changed pre-image must go stale: {racing:?}");
    };
    assert_eq!(stale.stale_fact_count(), 1);
    assert_eq!(title_of(&host, 1), "competitor");
    // The stale attempt published nothing, so the pulse did not move past the
    // competitor's commit.
    assert_eq!(Some(pulse.version()), after_competitor.get());
}
