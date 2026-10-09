use sha2::{Digest, Sha256};

use super::RecoveryStagingInput;

pub(super) fn exact_plan_commands(input: &RecoveryStagingInput) -> bool {
    let commands = input.staging.commands();
    if input.quiescence.staging_commands() != input.staging.scheduler_command_count() {
        return false;
    }
    let mut artifacts = std::collections::BTreeSet::new();
    commands.iter().enumerate().all(|(ordinal, command)| {
        command.ordinal() == ordinal as u64
            && !command.bytes().is_empty()
            && Sha256::digest(command.bytes()).as_slice() == command.payload_digest()
            && artifacts.insert((command.artifact(), command.offset()))
    })
}
