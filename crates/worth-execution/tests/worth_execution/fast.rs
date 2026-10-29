//! Fast integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "fast/byte_backing.rs"]
pub(crate) mod byte_backing;

#[path = "fast/charged_bytes_derive.rs"]
pub(crate) mod charged_bytes_derive;

#[path = "fast/decompose_pattern.rs"]
pub(crate) mod decompose_pattern;

#[path = "fast/fork_join_rounds.rs"]
pub(crate) mod fork_join_rounds;

#[path = "fast/keyed_kept_admission.rs"]
pub(crate) mod keyed_kept_admission;

#[path = "fast/map_admission.rs"]
pub(crate) mod map_admission;

#[path = "fast/memory_reservation.rs"]
pub(crate) mod memory_reservation;

#[path = "fast/oracle_bits.rs"]
pub(crate) mod oracle_bits;

#[path = "fast/partitioners.rs"]
pub(crate) mod partitioners;

#[path = "fast/reduce_pattern.rs"]
pub(crate) mod reduce_pattern;

#[path = "fast/scan_pattern.rs"]
pub(crate) mod scan_pattern;

#[path = "fast/sealed_request.rs"]
pub(crate) mod sealed_request;

#[path = "fast/serial_lineage.rs"]
pub(crate) mod serial_lineage;

#[path = "fast/work_ceiling.rs"]
pub(crate) mod work_ceiling;
