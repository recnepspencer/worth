#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "page_extent_recovery/mod.rs"]
pub(crate) mod page_extent_recovery;
#[allow(dead_code, unused_imports)]
#[path = "phase_six_publication_counters/mod.rs"]
pub(crate) mod phase_six_publication_counters;
#[allow(dead_code, unused_imports)]
#[path = "phase_six_reopen_faults/mod.rs"]
pub(crate) mod phase_six_reopen_faults;
#[allow(dead_code, unused_imports)]
#[path = "phase_three_support/mod.rs"]
pub(crate) mod phase_three_support;
#[allow(dead_code, unused_imports)]
#[path = "root_protocol_terminal_evidence/mod.rs"]
pub(crate) mod root_protocol_terminal_evidence;

#[path = "publication_faults.rs"]
pub(crate) mod publication_faults;
