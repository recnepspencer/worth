use worth_signal::facade::branch::ManagedSignalBranchReference;

fn main() {
    let _: ManagedSignalBranchReference =
        serde_json::from_str("{}").expect("transport data cannot mint owner authority");
}
