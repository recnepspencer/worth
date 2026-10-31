//! The decoder borrows the same original rows; clearing releases its share.
use super::{super::WorthQueryApplicationProjectionNode, OriginalProjection};
use std::sync::Arc;
use worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey;

pub(in crate::domain_computation::primary_graph::application_query) enum DisclosedRows {
    Owned(Vec<WorthQueryApplicationProjectionNode>),
    Shared(Arc<OriginalProjection>),
}

impl DisclosedRows {
    pub(in crate::domain_computation::primary_graph::application_query) fn rows(
        &self,
    ) -> &Vec<WorthQueryApplicationProjectionNode> {
        match self {
            Self::Owned(rows) => rows,
            Self::Shared(body) => &body.rows,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn clear(&mut self) {
        // Drop a shared reference without cloning or mutating the retained body.
        *self = Self::Owned(Vec::new());
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn retain(
        &mut self,
        disclosure: worth_execution::ExecutionArray<(Arc<ApplicationQueryResultSlotKey>, bool)>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Option<Arc<OriginalProjection>> {
        if !matches!(self, Self::Owned(_)) {
            return None;
        }
        let Self::Owned(rows) = std::mem::replace(self, Self::Owned(Vec::new())) else {
            unreachable!("the original one-shot tree was just checked")
        };
        let body = Arc::new(OriginalProjection::new(rows, disclosure, memory));
        *self = Self::Shared(Arc::clone(&body));
        Some(body)
    }
}
