//! The request a stage serves, compared by identity and never ordered.
use std::sync::Arc;

use super::{ExecutionResourceLease, LeaseNode};
use crate::authority::SerialRequest;

/// A serial request, or the lease node whose worker context a stage entered.
/// An entry naming one lives only while that stage borrows the request.
#[derive(Clone)]
pub(crate) enum StageOrigin {
    Serial(*const SerialRequest),
    Leased(Arc<LeaseNode>),
}

impl PartialEq for StageOrigin {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Serial(left), Self::Serial(right)) => std::ptr::eq(*left, *right),
            (Self::Leased(left), Self::Leased(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}

impl StageOrigin {
    pub(crate) fn serial(request: &SerialRequest) -> Self {
        Self::Serial(request)
    }
}

impl ExecutionResourceLease<'_> {
    pub(crate) fn stage_origin(&self) -> StageOrigin {
        StageOrigin::Leased(Arc::clone(&self.node))
    }
}
