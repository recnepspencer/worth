//! Runtime forwarding shims colocated with `application_query`.

mod ability_eligibility;
mod admission_finish;
mod application_entry;
mod authority_inventory;
mod batch_and_live_reads;
mod capability_eligibility;
mod continuation_and_retained_history;
mod correction_and_history_capture;
mod derived_collection_pair_maintenance;
mod derived_collection_pair_reconstruction;
mod history_open;
mod history_read;
mod managed_index;
mod one_shot_execution;
mod permission_and_identity_refresh;
mod source_expectations;
