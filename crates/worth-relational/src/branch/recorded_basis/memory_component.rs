use super::{
    copying, RecordedRelationalComponentReference, RelationalRecordedComponentRefusal as Refusal,
};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use std::sync::{Arc, Weak};
use worth_execution::ExecutionRequest;
pub(crate) mod live;
pub(crate) mod successor_runtime;
pub use worth_relational_branch::internal_port::branch::recorded_basis::memory_component::*;
