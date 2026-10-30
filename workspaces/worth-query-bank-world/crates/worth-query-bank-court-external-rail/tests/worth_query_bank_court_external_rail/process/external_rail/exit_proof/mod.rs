pub(crate) mod exactly_once;

pub(crate) mod fault_acknowledge_without_completing;

pub(crate) mod fault_complete_after_timeout;

pub(crate) mod fault_disappear_mid_dispatch;

pub(crate) mod fault_duplicate_acknowledgement;

pub(crate) mod fault_lost_response;

pub(crate) mod protocol_compatibility;

pub(crate) mod semantic_rejection;

pub(crate) mod separate_process;
