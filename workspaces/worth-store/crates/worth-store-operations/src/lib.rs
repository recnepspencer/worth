#![forbid(unsafe_code)]
//!
//! Backup/export custody readiness is not raw security metadata:
//!
//! ```compile_fail
//! use worth_store_operations::BackupExportCapsuleEmission;
//! use worth_store_security::StoreRawSecurityScopeDeclaration;
//!
//! let raw: StoreRawSecurityScopeDeclaration = todo!();
//! let _emission: BackupExportCapsuleEmission = raw;
//! ```
//!
//! Imported capsule observation is not readmitted readiness:
//!
//! ```compile_fail
//! use worth_store_offline_verifier::OfflineCustodyCapsuleObservation;
//! use worth_store_operations::BackupExportCustodyReadiness;
//!
//! let observed: OfflineCustodyCapsuleObservation = todo!();
//! let _readiness: BackupExportCustodyReadiness = observed;
//! ```
//!
//! Counters are evidence, not authority:
//!
//! ```compile_fail
//! use worth_store_operations::{
//!     BackupExportCapsuleEmission, BackupExportCustodyCounterSnapshot,
//! };
//!
//! let counters: BackupExportCustodyCounterSnapshot = todo!();
//! let _emission: BackupExportCapsuleEmission = counters;
//! ```
//!
//! Repair blast-radius readiness is not offline verifier evidence:
//!
//! ```compile_fail
//! use worth_store_offline_verifier::OfflineRepairBlastRadiusObservation;
//! use worth_store_operations::RepairBlastRadiusReadiness;
//!
//! let observed: OfflineRepairBlastRadiusObservation = todo!();
//! let _readiness: RepairBlastRadiusReadiness = observed;
//! ```
//!
//! Operator identity cannot stand in for repair-read physical readiness:
//!
//! ```compile_fail
//! use worth_store_operations::RepairBlastRadiusReadiness;
//! use worth_store_security::StoreOperatorIdentityClaim;
//!
//! let operator = StoreOperatorIdentityClaim::raw("operator-123");
//! let _readiness: RepairBlastRadiusReadiness = operator;
//! ```
//!
//! Repair readiness cannot stand in for operator authorization:
//!
//! ```compile_fail
//! use worth_store_operations::{RepairBlastRadiusReadiness, RepairOperatorAuthorization};
//!
//! let readiness: RepairBlastRadiusReadiness = todo!();
//! let _authorization: RepairOperatorAuthorization = readiness;
//! ```
//!
//! Durable control records are decoded only through the crate-private wire
//! schema; arbitrary bytes cannot manufacture a public domain record:
//!
//! ```compile_fail
//! use worth_store_operations::OperationalControlRecord;
//!
//! let _: OperationalControlRecord = OperationalControlRecord::decode(&[]).unwrap();
//! ```
//!
//! Restore authorization cannot satisfy repair admission:
//!
//! ```compile_fail
//! use worth_store_operations::{AuthorizedBackupRestorePlan, AuthorizedRepairPlan};
//!
//! let restore: AuthorizedBackupRestorePlan = todo!();
//! let _repair: AuthorizedRepairPlan = restore;
//! ```
//!
//! Restore authorization cannot satisfy PITR admission:
//!
//! ```compile_fail
//! use worth_store_operations::{
//!     AuthorizedBackupRestorePlan, AuthorizedPointInTimeRecoveryPlan,
//! };
//!
//! let restore: AuthorizedBackupRestorePlan = todo!();
//! let _pitr: AuthorizedPointInTimeRecoveryPlan = restore;
//! ```
//!
//! Restore authorization cannot satisfy rollback admission:
//!
//! ```compile_fail
//! use worth_store_operations::{AuthorizedBackupRestorePlan, AuthorizedRollbackPlan};
//!
//! let restore: AuthorizedBackupRestorePlan = todo!();
//! let _rollback: AuthorizedRollbackPlan = restore;
//! ```

mod authorization;
mod backup;
mod backup_export_custody_scheduler_demand;
mod boundary_projection;
#[cfg(feature = "certification-test-authority")]
pub mod certification_test_authority;
mod control_store;
mod facade;
pub mod layout_projection;
mod operational_audit;
mod operational_session;
mod owner_plan_dag;
#[cfg(test)]
mod phase_1_6_tests;
#[cfg(test)]
mod phase_7_13_tests;
mod repair;
mod repair_blast_radius_scheduler_demand;
mod replication_prep_scheduler_demand;
mod workflow;

pub use facade::*;
#[cfg(any(test, feature = "certification-test-authority"))]
pub mod certification_scenario;
