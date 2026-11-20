//! Explicit internal contract for Query execution schema layout lowering.

pub use crate::denial::{WorthQuerySchemaLayoutDenial, WorthQuerySchemaLayoutDenialKind};
pub use crate::outbox_layout::WorthQueryDispatchOutboxLayout;
pub use crate::workflow_actor_fields::{WorkflowActorFields, WorkflowEndingActorFields};
pub use crate::workflow_read_indexes::WorkflowReadIndexes;

pub mod schema_layout {
    pub use crate::schema_layout::*;
}

pub mod workflow_schema {
    pub use crate::workflow_schema::*;
}

/// Internal primary-graph layout lowering entrypoints for `worth-query-execution`.
/// Never re-exported on public execution facades or court doors.
pub mod lowering {
    pub use crate::schema_layout::lowering::{
        lower_layout, lower_layout_successor, lower_layout_with_original,
    };
}
