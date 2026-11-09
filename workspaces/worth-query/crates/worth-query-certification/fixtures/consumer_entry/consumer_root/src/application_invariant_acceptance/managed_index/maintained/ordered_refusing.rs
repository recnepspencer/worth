//! A distinct declared schema runs the same ordered Source under Refuse.
#![allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "This Refuse composition instantiates the complete ordinary schema and provider; each court uses only its own operations and observations."
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
#[path = "nested/shape.rs"]
mod shape;
use super::super::super::authentication;
use std::sync::Arc;
use worth_query_decl::facade::{application_query::*, application_schema::*};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};
const SCHEMA_OWNER: &str = "worth.query.consumer.ordered-event-refusal";
const SOURCE_COLD_POLICY: ApplicationManagedIndexColdSource =
    ApplicationManagedIndexColdSource::Refuse;
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

pub(super) fn missing_parent() {
    use std::sync::atomic::Ordering;
    let calls = Arc::new(prefix::Calls::default());
    let world = super::host::install_ordered_refusing(Arc::clone(&calls));
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
    let parent = request
        .query(query::FindPrefix {
            directory: "catalog".into(),
            prefix: "".into(),
        })
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request
        .query(query::FindPrefix {
            directory: "catalog".into(),
            prefix: "".into(),
        })
        .evict_index(&parent)
        .unwrap();
    let builds = calls.builds.load(Ordering::SeqCst);
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let result = request
        .advanced()
        .mutate(history::Edit::Append {
            rename: None,
            event: "E".into(),
            previous: "catalog".into(),
            introduced: None,
            retired: None,
        })
        .without_source()
        .idempotency(&3000u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("missing predecessor must refuse: {result:?}")
    };
    let source = denial
        .invariant_execution_failure()
        .unwrap()
        .managed_source_denial()
        .unwrap();
    assert!(source.source_unavailable(), "{source:?}");
    assert!(
        matches!(
            source,
            ManagedIndexSourcePreparationDenial::PredecessorUnavailable { .. }
        ),
        "{source:?}"
    );
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
}

fn source_graph_laws() -> ApplicationRelationGraphLaws {
    ApplicationRelationGraphLaws {
        cardinality: ApplicationRelationCardinality::new(None, Some(1), None, None, None, None),
        ..ApplicationRelationGraphLaws::unbounded()
    }
}
pub(super) fn stronger_law() {
    let world = super::host::install_ordered_refusing(Arc::new(prefix::Calls::default()));
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
    request
        .query(query::FindPrefix {
            directory: "catalog".into(),
            prefix: "".into(),
        })
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    for (key, event, previous, allowed) in
        [(4000u64, "E", "catalog", true), (4001, "F", "E", false)]
    {
        let result = request
            .advanced()
            .mutate(history::Edit::Append {
                rename: None,
                event: event.into(),
                previous: previous.into(),
                introduced: Some((event.to_lowercase(), format!("match-{event}"))),
                retired: None,
            })
            .without_source()
            .idempotency(&key)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        if allowed {
            assert!(result.result().is_some(), "{result:?}");
        } else {
            let WorthQueryApplicationMutationOutcome::Commit(
                WorthQueryApplicationUncommitted::Denied(denial),
            ) = result
            else {
                panic!("stronger law must refuse: {result:?}")
            };
            assert!(
                denial.kind() == WorthQueryApplicationCommitDenialKind::ProviderRejected
                    && denial.detail()
                        == Some("derived relation 'LiveItems' violated source-maximum"),
                "{denial:?}"
            );
        }
    }
}
