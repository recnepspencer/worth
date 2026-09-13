pub mod closeout;
pub mod memory_budget;
#[cfg(feature = "physical-isolation-fixtures")]
mod physical_source;
#[path = "drivers/fault_scheduler.rs"]
mod s4_fault_scheduler;
#[path = "drivers/storage_interposer.rs"]
mod s4_storage_interposer;
#[cfg(feature = "certification-world")]
pub mod wal_durability;
pub mod wal_tail;

#[cfg(feature = "physical-isolation-fixtures")]
pub use physical_source::{
    deterministic_checkpoint_plus_tail_source, deterministic_selected_compaction_product,
};
pub use s4_fault_scheduler::{FaultSchedulerDriver, ScheduledFault};
pub use s4_storage_interposer::{StorageBoundaryEvent, StorageBoundaryInterposerDriver};
