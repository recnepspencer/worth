//! One actual pair's late aggregate refusals cannot publish a current entry.

use std::cell::Cell;
use std::num::NonZeroUsize;
use std::sync::Arc;

use super::super::SceneLabel;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBatchReadDenial, WorthQueryApplicationQueryBatchAdmission as Batch,
    WorthQueryApplicationQueryBatchLimits as Limits,
    WorthQueryApplicationQueryBatchResourceDenial as Resource,
    WorthQueryManagedDerivedCollectionBatchRefreshDenial as Denial,
    WorthQueryManagedDerivedViewDenial as ViewDenial, WorthQueryManagedDerivedViewKey,
    WorthQueryManagedDerivedViewSnapshot,
};

fn batch(work: usize, bytes: usize) -> Batch {
    let nz = |value| NonZeroUsize::new(value).unwrap();
    Batch::new(Limits::new(nz(2), nz(work), nz(bytes), nz(4_096)))
}

pub(super) fn refuse_then_refresh(
    first_work: usize,
    first_memory: &Cell<usize>,
    projections: &Cell<usize>,
    snapshot: &WorthQueryManagedDerivedViewSnapshot<SceneLabel>,
    key: &WorthQueryManagedDerivedViewKey,
    mut refresh: impl FnMut(&Batch, &Batch) -> Result<Arc<SceneLabel>, Denial>,
) {
    let work_limited = batch(first_work, 262_144);
    let denial = refresh(&work_limited, &work_limited).err();
    assert!(
        matches!(
            denial,
            Some(Denial::Read(
                WorthQueryApplicationBatchReadDenial::Resource(Resource::WorkLimit { .. })
            ))
        ),
        "actual late read refusal: {denial:?}"
    );
    assert!(
        first_memory.get() > 0,
        "the first real result reached the second read"
    );
    assert_eq!(work_limited.observe().read_work_units(), first_work);
    assert_eq!(work_limited.observe().retained_bytes(), 0);
    assert_eq!(
        snapshot.get(key).err(),
        Some(ViewDenial::EntryRefreshRequired)
    );

    let memory_limited = batch(100_000, first_memory.get());
    assert!(matches!(
        refresh(&memory_limited, &memory_limited),
        Err(Denial::Read(
            WorthQueryApplicationBatchReadDenial::Resource(Resource::MemoryLimit { .. })
        ))
    ));
    assert_eq!(memory_limited.observe().retained_bytes(), 0);
    assert_eq!(
        snapshot.get(key).err(),
        Some(ViewDenial::EntryRefreshRequired)
    );

    let original = batch(100_000, 262_144);
    let foreign = batch(100_000, 262_144);
    assert!(matches!(
        refresh(&original, &foreign),
        Err(Denial::ForeignBatch)
    ));
    assert_eq!(projections.get(), 0);
    assert!(foreign.observe().read_work_units() > 0);
    assert_eq!(original.observe().retained_bytes(), 0);
    assert_eq!(foreign.observe().retained_bytes(), 0);
    assert_eq!(
        snapshot.get(key).err(),
        Some(ViewDenial::EntryRefreshRequired)
    );

    let complete = batch(100_000, 262_144);
    let refreshed = refresh(&complete, &complete).unwrap();
    assert_eq!(refreshed.0, "primary-changed");
    assert_eq!(projections.get(), 1);
    assert!(complete.observe().read_work_units() > first_work);
    assert!(complete.observe().peak_bytes() > first_memory.get());
    assert_eq!(complete.observe().retained_bytes(), 0);
    assert!(Arc::ptr_eq(
        &refreshed,
        &snapshot.get(key).unwrap().unwrap()
    ));
}

pub(super) fn deny_first(
    _: &String,
) -> Result<
    crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult<
        crate::domain_computation::primary_graph::tests::fixture::PublicScopedAccountSummaryQuery,
        crate::domain_computation::primary_graph::tests::fixture::AccountSummaryResult,
    >,
    ViewDenial,
> {
    Err(ViewDenial::QueryExecutionDenied)
}

pub(super) fn deny_second(
    _: &crate::domain_computation::primary_graph::tests::fixture::AccountSummaryResult,
) -> Result<
    crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult<
        crate::domain_computation::primary_graph::tests::fixture::PublicScopedAccountSummaryQuery,
        crate::domain_computation::primary_graph::tests::fixture::AccountSummaryResult,
    >,
    ViewDenial,
> {
    unreachable!("second read cannot run after first denial")
}
