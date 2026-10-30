use worth_relational::facade::snapshots::SnapshotHandle;

fn main() {
    let _: SnapshotHandle = rmp_serde::from_slice(&[]).unwrap();
}
