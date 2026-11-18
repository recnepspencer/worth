//! Every edit is judged by fresh execution and independent model call laws.
use super::differential::alphabet::{Kind, Model};
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as Cause,
    WorthQueryPartitionedComputationRunForTest as Run,
};

pub(super) const SEEDS: [u64; 2] = [0x9176_3c0b_5eed_0001, 0x9176_3c0b_5eed_0002];
const ROUNDS: usize = 1;
mod adoption;
mod bits;
pub(super) mod cancellation;
mod common_operation_handoff;
pub(in crate::checkpoint_recovery::computation_partition::oracle) mod correction_grant;
mod equal_output;
worth_execution::needs_os_threads! { mod interleaving; }
mod tree;
mod undo;
