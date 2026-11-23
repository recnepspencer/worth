//! Pre-effect native membership installation and exact predecessor write fences.
use super::*;
use crate::mvcc::RequiredOutputPublication as Publication;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use worth_execution::{ExecutionRequest, MapKernelContext};
pub(crate) mod complete;
mod predecessor;
mod producer_cascade;
pub(crate) use producer_cascade::CanonicalProducerRetirement;
mod relation_failure;
mod route_completion;
mod scope_dependents;
use relation_failure::relation_context;
pub(super) use relation_failure::relation_stop;
mod retained_origin;
use retained_origin::compare_retained_origin;
