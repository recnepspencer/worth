use worth_signal::facade::InstalledSignalConditionIdentity;

fn compare(
    current: &InstalledSignalConditionIdentity,
    candidate: &InstalledSignalConditionIdentity,
) -> bool {
    current == candidate
}

fn main() {}
