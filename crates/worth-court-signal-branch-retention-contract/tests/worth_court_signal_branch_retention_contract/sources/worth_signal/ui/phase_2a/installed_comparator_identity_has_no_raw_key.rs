use worth_signal::facade::InstalledSignalComparatorIdentity;

fn raw_key(identity: &InstalledSignalComparatorIdentity) -> u64 {
    identity.graph_instance_id()
}

fn main() {}
