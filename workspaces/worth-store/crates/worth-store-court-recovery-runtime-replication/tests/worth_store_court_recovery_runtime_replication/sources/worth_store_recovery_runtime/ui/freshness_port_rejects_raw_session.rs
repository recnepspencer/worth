use worth_store::physical_runtime::PhysicalRecoveryFreshnessPort;
use worth_store::physical_runtime::QualifiedRecoveryFilesystemMedia;

fn counterfeit_session(media: &QualifiedRecoveryFilesystemMedia) {
    let _ = PhysicalRecoveryFreshnessPort::admit(media, [7; 16]);
}

fn main() {}
