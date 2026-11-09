//! Faults cross the production manifest recovery and transaction read doors.
#[path = "cold_source.rs"]
mod cold_source;
use cold_source::MemberSource;
#[path = "cold_merge_history_matrix.rs"]
mod cold_merge_history_matrix;
#[path = "cold_read_matrix.rs"]
mod cold_read_matrix;
use crate::durability::data::{
    RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
};
use crate::durability::medium::{
    RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::tests::support::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

#[path = "cold_recovery_fixture.rs"]
mod cold_recovery_fixture;
use cold_recovery_fixture::checkpoint_fixture;
