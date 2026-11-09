//! Faults cross the production manifest recovery and transaction read doors.
#[path = "checkpoint_restart_bound.rs"]
mod checkpoint_restart_bound;
#[path = "cold_source.rs"]
mod cold_source;
use cold_source::MemberSource;
#[path = "cold_candidates.rs"]
mod cold_candidates;
#[path = "cold_historical_index_matrix.rs"]
mod cold_historical_index_matrix;
#[path = "cold_index_build_door.rs"]
mod cold_index_build_door;
#[path = "cold_index_checkpoint_door.rs"]
mod cold_index_checkpoint_door;
#[path = "cold_index_custody.rs"]
mod cold_index_custody;
#[path = "cold_index_fixture.rs"]
mod cold_index_fixture;
#[path = "cold_index_frontier.rs"]
mod cold_index_frontier;
#[path = "cold_index_matrix.rs"]
mod cold_index_matrix;
#[path = "cold_merge_history_matrix.rs"]
mod cold_merge_history_matrix;
#[path = "cold_merges.rs"]
mod cold_merges;
#[path = "cold_projection_scan_door.rs"]
mod cold_projection_scan_door;
#[path = "cold_read_door.rs"]
mod cold_read_door;
#[path = "cold_read_matrix.rs"]
mod cold_read_matrix;
#[path = "cold_reserved_checkpoint.rs"]
mod cold_reserved_checkpoint;
#[path = "cold_symbols.rs"]
mod cold_symbols;
#[path = "cold_tail_matrix.rs"]
mod cold_tail_matrix;
#[path = "cold_visibility.rs"]
mod cold_visibility;
use crate::durability::data::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalCheckpointArtifactPolicy,
    RelationalCheckpointMember, RelationalRecoveryStop,
};
use crate::durability::medium::{
    RelationalDurableCheckpointSource, RelationalDurableMemberLocator,
    RelationalDurableRecoverySource,
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::tests::support::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

#[path = "cold_isolation.rs"]
mod isolation;

#[path = "cold_recovery_fixture.rs"]
mod cold_recovery_fixture;
use cold_recovery_fixture::{
    checkpoint_fixture, fixture, fixture_from_runtime, fixture_with_history,
};
