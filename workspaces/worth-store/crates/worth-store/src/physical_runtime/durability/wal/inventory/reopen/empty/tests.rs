//! A lost checkpoint plus every lost WAL segment reopens as an empty WAL.
//! That is the whole history only under the first root.
use super::*;
use crate::physical_runtime::durability::{
    CheckpointCustodyCandidate, CheckpointCustodyOrigin, CleanReopenCheckpointCustody,
};

fn reopen_empty(first_root: bool) -> CheckpointCustodyOrigin {
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let inventory = empty_inventory(
        &paths::wal_directory(),
        PhysicalWalBindingReopenCutoff::GenerationZero,
        format,
    )
    .unwrap();
    let genesis = CleanReopenCheckpointCustody::TrustedGenesis { first_root };
    CheckpointCustodyCandidate::CleanReopen(genesis).admit(inventory.release_evidence())
}

#[test]
fn an_empty_wal_is_trusted_genesis_only_under_the_first_root() {
    assert_eq!(
        reopen_empty(true),
        CheckpointCustodyOrigin::CleanReopen(CleanReopenCheckpointCustody::TrustedGenesis {
            first_root: true
        })
    );
    assert_eq!(
        reopen_empty(false),
        CheckpointCustodyOrigin::ReopenRequiresC8
    );
}
