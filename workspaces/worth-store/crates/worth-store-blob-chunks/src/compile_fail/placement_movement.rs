//! Placement movement execution receipts cannot be synthesized from copied fields:
//! ```compile_fail
//! use worth_store_blob_chunks::ExecutedBlobPlacementMovementReceipt;
//!
//! let _forged = ExecutedBlobPlacementMovementReceipt {
//!     basis: todo!(),
//!     source_class: todo!(),
//!     target_class: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Published placement observations cannot be synthesized without execution:
//! ```compile_fail
//! use worth_store_blob_chunks::PublishedBlobPlacementObservation;
//!
//! let _forged = PublishedBlobPlacementObservation {
//!     basis: todo!(),
//!     placement_class: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! S.5 stable read receipts alone cannot satisfy S.7 movement execution:
//! ```compile_fail
//! use worth_store_blob_chunks::ExecutedBlobPlacementMovementReceipt;
//! use worth_store::physical_runtime::stability::StablePhysicalReadReceipt;
//!
//! fn requires_executed_movement(_: ExecutedBlobPlacementMovementReceipt) {}
//! let stable_read: StablePhysicalReadReceipt = todo!();
//! requires_executed_movement(stable_read);
//! ```
//! S.5 stable read receipts alone cannot satisfy S.7 movement read holds:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPlacementMovementReadHold;
//! use worth_store::physical_runtime::stability::StablePhysicalReadReceipt;
//!
//! fn requires_movement_read_hold(_: BlobPlacementMovementReadHold) {}
//! let stable_read: StablePhysicalReadReceipt = todo!();
//! requires_movement_read_hold(stable_read);
//! ```
//! S.5 movement read holds cannot mint movement verified-read evidence without streaming proof:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     AdmittedBlobPlacementMovementPlan, BlobMovementVerifiedReadEvidence,
//!     BlobPlacementMovementReadHold,
//! };
//!
//! let plan: AdmittedBlobPlacementMovementPlan = todo!();
//! let read_hold: BlobPlacementMovementReadHold = todo!();
//! let _read = BlobMovementVerifiedReadEvidence::from_movement_read_hold(&plan, read_hold);
//! ```
//! Store-owned physical execution receipts cannot be synthesized from copied interlocks:
//! ```compile_fail
//! use worth_store_physical_isolation::PhysicalPlacementMovementExecutionReceipt;
//!
//! let _forged = PhysicalPlacementMovementExecutionReceipt::<()> {
//!     intent: (),
//!     movement_interlock: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Raw stable digests cannot satisfy S.7 movement physical execution intent:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobPlacementMovementPhysicalExecutionIntent;
//! use worth_store_contracts::StableDigest;
//!
//! fn requires_movement_intent(_: BlobPlacementMovementPhysicalExecutionIntent) {}
//! let digest = StableDigest::new("copied").unwrap();
//! requires_movement_intent(digest);
//! ```
//! Lower physical movement execution authority is not an ordinary public marker:
//! ```compile_fail
//! use worth_store_physical_isolation::PhysicalPlacementMovementExecutionAuthority;
//!
//! let _marker = PhysicalPlacementMovementExecutionAuthority::store_owned();
//! ```
//! Store-owned S.7 movement execution adapter receipts cannot be synthesized
//! from copied plan fields:
//! ```compile_fail
//! use worth_store_blob_chunks::StoreOwnedPlacementMovementExecutionReceipt;
//!
//! let _forged = StoreOwnedPlacementMovementExecutionReceipt {
//!     basis: todo!(),
//!     source_class: todo!(),
//!     target_class: todo!(),
//!     movement_interlock: todo!(),
//! };
//! ```
//! The public Store execution marker cannot execute a movement plan directly:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     AdmittedBlobPlacementMovementPlan, StoreOwnedPlacementMovementExecution,
//! };
//!
//! let plan: AdmittedBlobPlacementMovementPlan = todo!();
//! let marker = StoreOwnedPlacementMovementExecution::store_owned();
//! let _ = marker.execute_physical_movement(&plan);
//! ```
//! The public Store execution marker cannot directly satisfy the execution receipt:
//! ```compile_fail
//! use worth_store_blob_chunks::{
//!     AdmittedBlobPlacementMovementPlan, StoreOwnedPlacementMovementExecution,
//! };
//!
//! let plan: AdmittedBlobPlacementMovementPlan = todo!();
//! let marker = StoreOwnedPlacementMovementExecution::store_owned();
//! let _ = plan.execute_with_receipt(marker);
//! ```
//! S.6 foreground reservations alone cannot satisfy S.7 movement execution:
//! ```compile_fail
//! use worth_store_blob_chunks::ExecutedBlobPlacementMovementReceipt;
//! use worth_store_io_scheduler::foreground_reservation::ForegroundReservationReceipt;
//!
//! fn requires_executed_movement(_: ExecutedBlobPlacementMovementReceipt) {}
//! let reservation: ForegroundReservationReceipt = todo!();
//! requires_executed_movement(reservation);
//! ```
//! Foundational performance receipts cannot publish placement observations:
//! ```compile_fail
//! use worth_foundational::{
//!     FoundationalAuthoritativePerformanceClaim, FoundationalCounterBackedPerformanceReceipt,
//! };
//! use worth_store_blob_chunks::PublishedBlobPlacementObservation;
//!
//! fn requires_published_observation(_: PublishedBlobPlacementObservation) {}
//! let receipt: FoundationalCounterBackedPerformanceReceipt<
//!     FoundationalAuthoritativePerformanceClaim,
//! > = todo!();
//! requires_published_observation(receipt);
//! ```
