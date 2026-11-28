//! Restored public-boundary doctests for items moved into split crates.

pub mod contracts_01;
pub mod contracts_02;
pub mod contracts_03;
pub mod contracts_04;
pub mod contracts_05;
pub mod contracts_06;
pub mod retired_authority;

#[doc = include_str!("../../../worth-query-execution-runtime/src/domain_computation/primary_graph/application_contribution/request_execution/advancement/phase_request_contracts.md")]
pub struct PhaseRequestBoundaryContracts;

#[doc = include_str!("../../../worth-query-execution-query/src/domain_computation/primary_graph/application_query/managed_index/source_row.md")]
pub struct ManagedIndexSourceRowBoundaryContracts;
