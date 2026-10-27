//! One World parent OR one Native chunk; neither operation installs the aggregate position.
use std::{num::NonZeroUsize, sync::Arc};
use worth_query_declaration::facade::{
    application_query::ApplicationHistoryQueryBinding, application_schema::ApplicationSchema,
};
use worth_relational::facade::{
    history::{RelationalCanonicalRecordHistoryCursor, RelationalRecordHistoryPageLimits},
    runtime::RelationalRuntime,
};
use worth_runtime_world::facade::{CompositeCommitParent, ProductExactHistoryCursor};

use super::{
    cursor::{HistoryPosition, RetainedWorldCursor, RetainedWorldPage},
    storage, WorthQueryApplicationHistoryCursor as Cursor,
    WorthQueryApplicationHistoryDenial as Denial, WorthQueryApplicationHistoryPage as Page,
    WorthQueryApplicationHistoryPageLimits as Limits,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(super) struct HistoryTransition {
    pub(super) page: Page,
    pub(super) position: HistoryPosition,
}

pub(super) struct WorldTransition {
    world: Arc<RetainedWorldPage>,
    after: Arc<RetainedWorldCursor>,
    newly_owned_bytes: usize,
}

pub(super) fn prepare_world<Schema: ApplicationSchema>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    original: &RetainedWorldCursor,
    limits: Limits,
) -> Result<WorldTransition, Denial> {
    // Admit the maximum new position and possible exact Native source before either allocation.
    let wrappers = storage::add(&[
        storage::arc_bytes::<RetainedWorldPage>(),
        storage::arc_bytes::<RetainedWorldCursor>(),
        size_of::<Page>(),
    ])?;
    storage::require(
        storage::add(&[
            wrappers,
            ProductExactHistoryCursor::transition_backing_bytes(),
            RelationalCanonicalRecordHistoryCursor::owned_source_backing_bytes(),
        ])?,
        limits.maximum_owned_bytes.get(),
    )?;
    let prepared = application
        .product_runtime
        .prepare_exact_product_history_page(execution, &original.cursor)
        .map_err(Denial::World)?;
    let parent_bytes = if prepared.is_complete() {
        0
    } else {
        ProductExactHistoryCursor::transition_backing_bytes()
    };
    let (page, after) = original
        .cursor
        .accept_page(prepared)
        .map_err(Denial::World)?;
    Ok(WorldTransition {
        world: RetainedWorldPage::retain(execution, page)?,
        after: RetainedWorldCursor::retain(execution, after)?,
        newly_owned_bytes: storage::add(&[wrappers, parent_bytes])?,
    })
}

impl WorldTransition {
    /// Called only after fresh SubjectRead and metadata admission on the genuine graph session.
    pub(super) fn finish<
        Schema: ApplicationSchema,
        History: ApplicationHistoryQueryBinding<Schema>,
    >(
        self,
        runtime: &RelationalRuntime,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        cursor: &Cursor<Schema, History>,
    ) -> Result<HistoryTransition, Denial> {
        let Some(commit) = self.world.page.commit() else {
            return Ok(complete());
        };
        let basis = commit.basis().relational_basis();
        let native_commit = match commit.parent() {
            CompositeCommitParent::Root => basis.observation().commit_id(),
            CompositeCommitParent::Ordinary(_) => commit
                .relational_publication_identity()
                .map(|identity| identity.commit_id()),
        };
        let position = match (native_commit, cursor.source.subjects.owner.native.as_ref()) {
            (Some(commit), Some(selection)) => {
                let selected = runtime
                    .select_exact_commit(&basis.observation(), commit)
                    .into_outcome()
                    .map_err(|_| Denial::SubjectMismatch)?;
                let native = runtime
                    .record_aspect_history_cursor_from_selection(execution, selected, selection)
                    .map_err(Denial::Native)?;
                HistoryPosition::Native {
                    world: Arc::clone(&self.world),
                    after_world: self.after,
                    cursor: native,
                }
            }
            _ if self.world.page.is_complete() => HistoryPosition::Complete,
            _ => HistoryPosition::World(self.after),
        };
        let native_bytes = if matches!(&position, HistoryPosition::Native { .. }) {
            RelationalCanonicalRecordHistoryCursor::owned_source_backing_bytes()
        } else {
            0
        };
        Ok(HistoryTransition {
            page: Page {
                source_visits: self.world.page.source_visits(),
                complete: matches!(&position, HistoryPosition::Complete),
                newly_owned_bytes: storage::add(&[self.newly_owned_bytes, native_bytes])?,
                world: Some(self.world),
                native: None,
                authorization_work: Default::default(),
            },
            position,
        })
    }
}

pub(super) fn prepare_native(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    world: &Arc<RetainedWorldPage>,
    after_world: &Arc<RetainedWorldCursor>,
    cursor: &RelationalCanonicalRecordHistoryCursor,
    limits: Limits,
) -> Result<HistoryTransition, Denial> {
    let remaining = limits
        .maximum_owned_bytes
        .get()
        .checked_sub(size_of::<Page>())
        .and_then(NonZeroUsize::new)
        .ok_or(Denial::StorageLimitExceeded)?;
    let prepared = cursor
        .prepare_next_page(
            runtime,
            execution,
            RelationalRecordHistoryPageLimits {
                maximum_entries: limits.maximum_entries,
                maximum_source_visits: limits.maximum_source_visits,
                maximum_owned_bytes: remaining,
            },
        )
        .map_err(Denial::Native)?;
    let (native, next) = cursor.accept_page(prepared).map_err(Denial::Native)?;
    let complete = native.is_complete() && world.page.is_complete();
    let position = if complete {
        HistoryPosition::Complete
    } else if native.is_complete() {
        HistoryPosition::World(Arc::clone(after_world))
    } else {
        HistoryPosition::Native {
            world: Arc::clone(world),
            after_world: Arc::clone(after_world),
            cursor: next,
        }
    };
    Ok(HistoryTransition {
        page: Page {
            complete,
            source_visits: native.source_visits(),
            newly_owned_bytes: storage::add(&[size_of::<Page>(), native.owned_backing_bytes()])?,
            world: Some(Arc::clone(world)),
            native: Some(native),
            authorization_work: Default::default(),
        },
        position,
    })
}

pub(super) fn complete() -> HistoryTransition {
    HistoryTransition {
        page: Page {
            world: None,
            native: None,
            complete: true,
            source_visits: 0,
            newly_owned_bytes: size_of::<Page>(),
            authorization_work: Default::default(),
        },
        position: HistoryPosition::Complete,
    }
}
