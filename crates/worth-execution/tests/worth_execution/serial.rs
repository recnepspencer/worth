//! Serial integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "serial/array_backing.rs"]
pub(crate) mod array_backing;

#[path = "serial/byte_backing.rs"]
pub(crate) mod byte_backing;

#[path = "serial/charged_bytes_derive.rs"]
pub(crate) mod charged_bytes_derive;

#[path = "serial/decompose_frontier.rs"]
pub(crate) mod decompose_frontier;

#[path = "serial/decompose_pattern.rs"]
pub(crate) mod decompose_pattern;

#[path = "serial/equivalence_contracts.rs"]
pub(crate) mod equivalence_contracts;

#[path = "serial/fork_join_rounds.rs"]
pub(crate) mod fork_join_rounds;

#[path = "serial/memory_reservation.rs"]
pub(crate) mod memory_reservation;

#[path = "serial/nested_reduction.rs"]
pub(crate) mod nested_reduction;

#[path = "serial/oracle_bits.rs"]
pub(crate) mod oracle_bits;

#[path = "serial/partitioners.rs"]
pub(crate) mod partitioners;

#[path = "serial/partitioner_checked.rs"]
pub(crate) mod partitioner_checked;

#[path = "serial/phase_two_certification_repair.rs"]
pub(crate) mod phase_two_certification_repair;

#[path = "serial/phase_two_neutral_certification.rs"]
pub(crate) mod phase_two_neutral_certification;

#[path = "serial/process_zero_memory.rs"]
pub(crate) mod process_zero_memory;

#[path = "serial/public_boundaries.rs"]
pub(crate) mod public_boundaries;

#[path = "serial/public_map.rs"]
pub(crate) mod public_map;

#[path = "serial/public_nested_regressions.rs"]
pub(crate) mod public_nested_regressions;

#[path = "serial/reduce_frontier.rs"]
pub(crate) mod reduce_frontier;

#[path = "serial/reduce_pattern.rs"]
pub(crate) mod reduce_pattern;

#[path = "serial/scan_pattern.rs"]
pub(crate) mod scan_pattern;

#[path = "serial/sealed_request.rs"]
pub(crate) mod sealed_request;

#[path = "serial/worker_authority_lifetime.rs"]
pub(crate) mod worker_authority_lifetime;

#[path = "serial/work_ceiling.rs"]
pub(crate) mod work_ceiling;
