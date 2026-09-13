#![forbid(unsafe_code)]
// Owner-free dependency graphs retain passive media vocabulary but cannot
// activate the filesystem-owner graph. All-feature CI still denies dead code.
#![cfg_attr(not(feature = "store-runtime-owner"), allow(dead_code))]
//! Backend/media capability witnesses cannot be synthesized from raw fields:
//! ```compile_fail
//! use worth_store_physical_backend::AdmittedBackendCapabilityWitness;
//! let _forged = AdmittedBackendCapabilityWitness {
//!     profile: todo!(),
//!     evidence_class: todo!(),
//!     support: todo!(),
//!     media_assumptions: todo!(),
//!     rebind_triggers: todo!(),
//!     confidence_limits: todo!(),
//! };
//! ```
//! Claim witnesses cannot be constructed by copying backend labels or rows:
//! ```compile_fail
//! use worth_store_physical_backend::BackendCapabilityClaimWitness;
//! let _forged = BackendCapabilityClaimWitness {
//!     profile: todo!(),
//!     evidence_class: todo!(),
//!     kind: todo!(),
//! };
//! ```
//! Raw probe rows or terminal projections cannot stand in for Store-owned
//! capability evidence at the public admission boundary:
//! ```compile_fail
//! use worth_store_physical_backend::{
//!     BackendCapabilityAdmissionRequest, BackendCapabilitySupportSet,
//!     BackendMediaAssumptionSet, BackendRebindTriggers, BackendTargetProfile,
//! };
//! struct RawProbeRow;
//! struct TerminalProjection;
//! let _request = BackendCapabilityAdmissionRequest::new(
//!     BackendTargetProfile::PosixFileFsyncDirSync,
//!     RawProbeRow,
//!     BackendCapabilitySupportSet::all_supported(),
//!     BackendMediaAssumptionSet::platform_file_defaults(),
//!     TerminalProjection,
//! );
//! ```
//! Backend residue observations cannot be minted from raw caller fields:
//! ```compile_fail
//! use worth_store_physical_backend::{
//!     BlobBackendResidueObservation, BlobBackendResidueObservationKind,
//! };
//!
//! let _forged = BlobBackendResidueObservation::observed(
//!     BlobBackendResidueObservationKind::OrphanedPlacementResidue,
//!     "copied-object-key",
//! );
//! ```
//! Backend residue evidence cannot be minted from copied tokens plus a copied
//! capability claim:
//! ```compile_fail
//! use worth_store_physical_backend::{
//!     BackendCapabilityClaimWitness, BlobBackendResidueObservation,
//!     BlobBackendResidueObservationKind,
//! };
//! let capability: BackendCapabilityClaimWitness = todo!();
//! let _forged = BlobBackendResidueObservation::from_store_backend_residue_scan(
//!     capability,
//!     BlobBackendResidueObservationKind::OrphanedPlacementResidue,
//!     "copied-object-key",
//! );
//! ```
//! Backend manifest evidence cannot be minted from copied digest/scope fields:
//! ```compile_fail
//! use worth_store_physical_backend::{
//!     BackendCapabilityClaimWitness, BlobPhysicalManifestObservation,
//! };
//! let capability: BackendCapabilityClaimWitness = todo!();
//! let _forged = BlobPhysicalManifestObservation::from_store_backend_manifest_traversal(
//!     capability, "copied-digest", 1, "copied-digest", 1, todo!(), true,
//! );
//! ```
//! Queue execution completion cannot be minted from ordinary caller-supplied
//! fields in production builds:
//! ```compile_fail
//! use worth_store_physical_backend::BackendQueueExecutionCompletion;
//! let _forged = BackendQueueExecutionCompletion::completed(todo!(), todo!());
//! ```
//! Queue execution tickets are backend-issued authority; ordinary callers cannot
//! construct them from replay bindings:
//! ```compile_fail
//! use worth_store_physical_backend::BackendQueueExecutionTicket;
//! let _forged = BackendQueueExecutionTicket::from_backend_authority(todo!(), todo!());
//! ```
//! Ordinary callers cannot issue queue execution tickets from caller-supplied
//! replay bindings and a borrowed capability witness in production builds:
//! ```compile_fail
//! use worth_store_physical_backend::{
//!     AdmittedBackendCapabilityWitness, BackendQueueExecutionAdaptation,
//!     BackendQueueExecutionAuthority, BackendQueueExecutionPlanBinding,
//! };
//! fn worth_ticket(
//!     binding: BackendQueueExecutionPlanBinding,
//!     witness: &AdmittedBackendCapabilityWitness,
//! ) {
//!     let _ = BackendQueueExecutionAuthority::store_owned().issue_ticket(
//!         binding,
//!         witness,
//!         BackendQueueExecutionAdaptation::None,
//!     );
//! }
//! ```
//! Ordinary callers cannot combine public observations and a backend
//! implementation into queue completion authority:
//! ```compile_fail
//! use worth_store_physical_backend::{
//!     BackendQueueExecutionObservedCounters, BackendQueueExecutionSession, PhysicalReference,
//!     PhysicalStoreBackend, StoreOwnedBackendQueueExecution,
//! };
//! struct Dummy;
//! impl PhysicalStoreBackend for Dummy {
//!     type Error = ();
//!     fn append_framed_record(&mut self, _: &[u8]) -> Result<PhysicalReference, Self::Error> {
//!         Err(())
//!     }
//!     fn read_framed_record(&self, _: PhysicalReference) -> Result<Vec<u8>, Self::Error> {
//!         Err(())
//!     }
//! }
//! let mut backend = Dummy;
//! let observations = BackendQueueExecutionObservedCounters::new();
//! let authority = StoreOwnedBackendQueueExecution { _private: () };
//! let mut session = BackendQueueExecutionSession::for_store_backend(&mut backend, authority);
//! let _ = session.complete_after_append(todo!(), todo!(), todo!(), b"payload", observations);
//! ```
//! Production queue execution sessions require Store-owned execution authority;
//! ordinary backend implementations cannot open a completion-minting session:
//! ```compile_fail
//! use worth_store_physical_backend::{
//!     BackendQueueExecutionSession, PhysicalReference, PhysicalStoreBackend,
//!     StoreOwnedBackendQueueExecution,
//! };
//! struct Dummy;
//! impl PhysicalStoreBackend for Dummy {
//!     type Error = ();
//!     fn append_framed_record(&mut self, _: &[u8]) -> Result<PhysicalReference, Self::Error> {
//!         Err(())
//!     }
//!     fn read_framed_record(&self, _: PhysicalReference) -> Result<Vec<u8>, Self::Error> {
//!         Err(())
//!     }
//! }
//! let mut backend = Dummy;
//! let authority = StoreOwnedBackendQueueExecution { _private: () };
//! let _session = BackendQueueExecutionSession::for_store_backend(&mut backend, authority);
//! ```
//! Store access policy admission cannot be forged from public fields:
//! ```compile_fail
//! use worth_store_physical_backend::AdmittedAccessPolicy;
//! let _forged = AdmittedAccessPolicy {
//!     request: todo!(),
//!     capability: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Store access execution receipts cannot be constructed without an admitted
//! policy completing through the Store-owned receipt path:
//! ```compile_fail
//! use worth_store_physical_backend::AccessPolicyExecutionReceipt;
//! let _forged = AccessPolicyExecutionReceipt {
//!     policy: todo!(),
//!     counters: todo!(),
//! };
//! ```
//! Store access prerequisite proofs cannot be field-forged downstream:
//! ```compile_fail
//! use worth_store_physical_backend::{
//!     AccessPolicyBufferLifecycle, AccessPolicyBufferLifecycleKind,
//! };
//! let _forged = AccessPolicyBufferLifecycle {
//!     kind: AccessPolicyBufferLifecycleKind::PinnedPhysicalLease,
//! };
//! ```
//! ```compile_fail
//! let _forged = worth_store_physical_backend::AccessPolicyBufferLifecycle { kind: AccessPolicyBufferLifecycleKind::PinnedPhysicalLease };
//! ```
//! ```compile_fail
//! use worth_store_physical_backend::DirectIoAlignmentRequirement;
//! let _forged = DirectIoAlignmentRequirement {
//!     page_aligned: true,
//!     sector_aligned: true,
//!     buffer_lifetime_stable: true,
//! };
//! ```
//! ```compile_fail
//! use worth_store_physical_backend::MmapFaultPosture;
//! let _forged = MmapFaultPosture {
//!     fault: todo!(),
//!     writeback: todo!(),
//!     visibility: todo!(),
//!     truncate: todo!(),
//!     punch_hole: todo!(),
//! };
//! ```
//! ```compile_fail
//! use worth_store_physical_backend::MixedAccessCoherenceBasis;
//! let _forged = MixedAccessCoherenceBasis {
//!     transition: todo!(),
//!     reference: todo!(),
//!     security_scope: todo!(),
//!     invalidation: todo!(),
//!     writeback: todo!(),
//! };
//! ```
//! Store access execution authority cannot be opened without Store authority:
//! ```compile_fail
//! use worth_store_physical_backend::StoreOwnedAccessPolicyExecution;
//! let _forged = StoreOwnedAccessPolicyExecution { _private: () };
//! ```
extern crate self as worth_store_physical_backend;

mod access_policy;
mod backup_materialization;
mod directory_durability;
mod durability_profile;
mod execution;
pub mod external_recovery_compile_fail;
mod facade;
mod filesystem_media;
mod heavy_fixture;
mod io_capability;
mod media_topology;
mod offline_media;
mod operation;
mod operation_boundary;
mod operational_control;
#[cfg(not(feature = "certification-test-authority"))]
pub mod ordinary_authority_compile_fail;
mod placement_observation;
mod recovery_durability;
#[cfg(feature = "recovery-runtime-owner")]
mod recovery_media;
mod recovery_staging;
mod storage_boundary_control;
pub(crate) use execution::queue::BackendQueueExecutionAuthority;
pub use facade::*;
