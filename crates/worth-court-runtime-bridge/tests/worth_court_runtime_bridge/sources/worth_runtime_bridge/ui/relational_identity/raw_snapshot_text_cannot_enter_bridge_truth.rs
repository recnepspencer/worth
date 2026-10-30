use worth_runtime_bridge::facade::TruthSnapshotIdentity;

fn main() {
    let _identity =
        TruthSnapshotIdentity::from_relational_snapshot("relational-snapshot:7:version:7");
}
