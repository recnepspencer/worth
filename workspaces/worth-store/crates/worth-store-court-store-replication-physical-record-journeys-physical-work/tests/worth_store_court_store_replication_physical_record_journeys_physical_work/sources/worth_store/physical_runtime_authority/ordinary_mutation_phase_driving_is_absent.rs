use worth_proof::NonEmpty;
use worth_store::physical_runtime::{PhysicalRecordSubmission, PreparedPhysicalMutation};

fn ordinary_caller_cannot_drive_wal(
    submission: PhysicalRecordSubmission,
    prepared: PreparedPhysicalMutation,
) {
    let _ = submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new()));
}

fn main() {}
