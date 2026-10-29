//! Every node-consuming public operation shares the failure-class contract.
use super::cold_lineage_fixture::reopened;
use super::cold_lineage_nodes::NodeDoor;
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

#[cfg(test)]
#[path = "cold_lineage_matrix/exhaustive.rs"]
mod exhaustive;
