//! Independent selected-media rejoin for a post-checkpoint WAL release.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedPendingWalReleaseCustody,
    VerifiedSelectedTierEpochCustody,
};

use super::{
    tier, wal_fate, wal_inventory, SelectedControlMediaFingerprint,
    SelectedMediaRejoinDenial as Denial, SelectedWalMediaFingerprint, MAX_CHECKPOINT_BYTES,
    MAX_CLEANUP_SAMPLE_BYTES, MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, IntegrityAdmittedRecoveryWalFrameView,
    PhysicalRecoveryAllocationAdmission, PhysicalRecoveryCoordination,
    PhysicalRecoveryFreshnessPort,
};

#[path = "pending_wal_release/observation.rs"]
mod observation;
pub(in crate::physical_runtime::recovery_construction) use observation::observe_claim;

#[path = "pending_wal_release/addressed_root.rs"]
mod addressed_root;
#[path = "pending_wal_release/controls.rs"]
mod controls;
#[path = "pending_wal_release/delta.rs"]
pub(super) mod delta;
#[path = "pending_wal_release/head_effect_media.rs"]
mod head_effect_media;
#[path = "pending_wal_release/head_v14.rs"]
mod head_v14;
#[path = "pending_wal_release/historical_chain.rs"]
mod historical_chain;
#[path = "pending_wal_release/historical_first.rs"]
mod historical_first;
#[path = "pending_wal_release/lineage.rs"]
mod lineage;
#[path = "pending_wal_release/ordered_history.rs"]
mod ordered_history;
#[path = "pending_wal_release/ordered_released.rs"]
mod ordered_released;
#[path = "pending_wal_release/ordered_walk.rs"]
pub(super) mod ordered_walk;
#[path = "pending_wal_release/ordinary_member.rs"]
mod ordinary_member;
#[path = "pending_wal_release/resident_budget.rs"]
mod resident_budget;
#[path = "pending_wal_release/selection.rs"]
pub(super) mod selection;
#[path = "pending_wal_release/topology.rs"]
mod topology;
