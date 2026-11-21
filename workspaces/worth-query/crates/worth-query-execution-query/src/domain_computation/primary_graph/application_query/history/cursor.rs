//! Private aggregate positions compose genuine immutable Native/World successors.
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_query::ApplicationHistoryQueryBinding, application_schema::ApplicationSchema,
};
use worth_query_execution_runtime::facade::ports::ProductRuntimeBranchLifecycleInternalPort as _;
use worth_relational::facade::history::RelationalCanonicalRecordHistoryCursor;
use worth_runtime_world::facade::{ProductExactHistoryCursor, ProductExactHistoryPage};

use super::{
    storage, WorthQueryApplicationHistoryDenial as Denial,
    WorthQueryCapturedApplicationHistorySubjects,
};

/// A pinned history position. It retains provenance and position, never read permission.
pub struct WorthQueryApplicationHistoryCursor<
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
> {
    pub(crate) source: Arc<HistoryCursorSource<Schema, History>>,
    pub(crate) position: HistoryPosition,
}

pub struct HistoryCursorSource<
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
> {
    pub subjects: WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
    pub target: crate::basis::WorthQueryProductObservationLease,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub enum HistoryPosition {
    World(Arc<RetainedWorldCursor>),
    Native {
        world: Arc<RetainedWorldPage>,
        after_world: Arc<RetainedWorldCursor>,
        cursor: RelationalCanonicalRecordHistoryCursor,
    },
    Complete,
}

pub struct RetainedWorldCursor {
    pub cursor: ProductExactHistoryCursor,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub struct RetainedWorldPage {
    pub page: ProductExactHistoryPage,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RetainedWorldCursor {
    pub fn retain(
        execution: worth_execution::ExecutionRequest<'_, '_>,
        cursor: ProductExactHistoryCursor,
    ) -> Result<Arc<Self>, Denial> {
        let memory = storage::retain(execution, storage::arc_bytes::<Self>())?;
        Ok(Arc::new(Self {
            cursor,
            _memory: memory,
        }))
    }
}
impl RetainedWorldPage {
    pub fn retain(
        execution: worth_execution::ExecutionRequest<'_, '_>,
        page: ProductExactHistoryPage,
    ) -> Result<Arc<Self>, Denial> {
        let memory = storage::retain(execution, storage::arc_bytes::<Self>())?;
        Ok(Arc::new(Self {
            page,
            _memory: memory,
        }))
    }
}

impl<Schema: ApplicationSchema, History: ApplicationHistoryQueryBinding<Schema>>
    WorthQueryApplicationHistoryCursor<Schema, History>
{
    pub(crate) fn open(
        execution: worth_execution::ExecutionRequest<'_, '_>,
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        subjects: WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        target: crate::basis::WorthQueryProductObservationLease,
        maximum_owned_bytes: usize,
    ) -> Result<Self, Denial> {
        let source_bytes = storage::arc_bytes::<HistoryCursorSource<Schema, History>>();
        let has_subjects = subjects.owner.native.is_some();
        let bytes = if has_subjects {
            storage::add(&[
                source_bytes,
                storage::arc_bytes::<RetainedWorldCursor>(),
                ProductExactHistoryCursor::opening_backing_bytes(),
            ])?
        } else {
            source_bytes
        };
        storage::require(bytes, maximum_owned_bytes)?;
        let memory = storage::retain(execution, source_bytes)?;
        let position = if has_subjects {
            let cursor = application
                .runtime
                .product_runtime
                .open_exact_product_history(execution, &target)
                .map_err(Denial::World)?;
            HistoryPosition::World(RetainedWorldCursor::retain(execution, cursor)?)
        } else {
            HistoryPosition::Complete
        };
        Ok(Self {
            source: Arc::new(HistoryCursorSource {
                subjects,
                target,
                _memory: memory,
            }),
            position,
        })
    }

    pub(crate) fn successor(&self, position: HistoryPosition) -> Self {
        Self {
            source: Arc::clone(&self.source),
            position,
        }
    }
}
