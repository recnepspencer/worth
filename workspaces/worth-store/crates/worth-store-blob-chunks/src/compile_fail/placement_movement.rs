//! Movement planning is not physical movement execution.
//!
//! A lower read-plan completion alone cannot supply a migration interlock:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPlacementMovementReadPlanBasis;
//! use worth_store_physical_isolation::PhysicalReadPlanCompletionReceipt;
//! fn requires_plan(_: BlobPlacementMovementReadPlanBasis) {}
//! let completion: PhysicalReadPlanCompletionReceipt = todo!();
//! requires_plan(completion);
//! ```
//!
//! A planning result has no physical execution or publication method:
//! ```compile_fail
//! use worth_store_blob_chunks::AdmittedBlobPlacementMovementPlan;
//! let plan: AdmittedBlobPlacementMovementPlan = todo!();
//! let _ = plan.execute_with_receipt(todo!());
//! ```
//!
//! A copied digest cannot construct the plan's private basis:
//! ```compile_fail
//! use worth_store_blob_chunks::AdmittedBlobPlacementMovementPlan;
//! let _plan = AdmittedBlobPlacementMovementPlan {
//!     basis: todo!(),
//!     source_class: todo!(),
//!     target_class: todo!(),
//!     read_plan: todo!(),
//!     cold_outcome: todo!(),
//!     counters: todo!(),
//! };
//! ```
