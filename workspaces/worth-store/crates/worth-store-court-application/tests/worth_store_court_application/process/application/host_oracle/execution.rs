//! Shared script dispatch runs actual production operations on each Host.
use super::protocol::{Action, Disposition};
use super::*;
use host_observation::{
    projection::HeadProjection,
    receipts::{self, MutationProjection},
    requests, OracleHome,
};
use std::collections::HashMap;

pub(super) fn mutation(
    home: &OracleHome,
    command: &HostCommand,
    before: &mut HashMap<u64, HeadProjection>,
) -> Option<MutationProjection> {
    let (key, value, lost, failure, abandon) = match command {
        HostCommand::Mutate { key, value, .. } => (*key, *value, false, false, false),
        HostCommand::MoveProduct { value, .. } => (None, *value, false, false, false),
        HostCommand::Retry { key, value } => (Some(*key), *value, false, false, false),
        HostCommand::LoseResponse { key, value } => (Some(*key), *value, true, false, false),
        HostCommand::SettlementFailure { key, value } => (Some(*key), *value, false, true, false),
        HostCommand::AbandonCaller { key, value } => (Some(*key), *value, false, false, true),
        HostCommand::Open
        | HostCommand::Initialize
        | HostCommand::CreateBranch(_)
        | HostCommand::ContinueCustody
        | HostCommand::ReleaseCustody
        | HostCommand::Checkpoint(_)
        | HostCommand::Close
        | HostCommand::Reopen
        | HostCommand::ProgramTransition { .. }
        | HostCommand::ResumeTransition => return None,
    };
    let head =
        HeadProjection::observe(&home.runtime().runtime().application_oracle_head_for_test());
    let original = key
        .and_then(|key| before.get(&key))
        .unwrap_or(&head)
        .clone();
    let outcome = if failure {
        host_interruption::publication::settlement_failure(home.runtime(), key.unwrap(), value)
    } else if abandon {
        host_interruption::publication::abandon_after_owner(home.runtime(), key.unwrap(), value)
    } else {
        requests::set(home.runtime(), key, value).expect("HOST_SCRIPT_BOUNDARY_BLOCKED: full active vocabulary requires a genuine unkeyed Host mutation entry")
    };
    let observed = receipts::observe(&outcome, &original);
    if observed.fate == receipts::MutationFate::Committed {
        if let Some(key) = key {
            before.entry(key).or_insert(head);
        }
    }
    // The observer may see state; the application response is deliberately lost.
    if lost {
        None
    } else {
        Some(observed)
    }
}
pub(super) fn disposition(home: &OracleHome, command: &HostCommand) -> Option<Disposition> {
    match command {
        HostCommand::ContinueCustody => Some(Disposition::Continued(
            host_interruption::publication::continue_custody(home.runtime())
                .unwrap()
                .into_iter()
                .map(|c| c.actions().iter().copied().map(Action::observe).collect())
                .collect(),
        )),
        HostCommand::ReleaseCustody => Some(Disposition::Released(
            host_interruption::publication::release_custody(home.runtime()).unwrap(),
        )),
        HostCommand::Open
        | HostCommand::Initialize
        | HostCommand::Mutate { .. }
        | HostCommand::LoseResponse { .. }
        | HostCommand::Retry { .. }
        | HostCommand::CreateBranch(_)
        | HostCommand::MoveProduct { .. }
        | HostCommand::SettlementFailure { .. }
        | HostCommand::AbandonCaller { .. }
        | HostCommand::Checkpoint(_)
        | HostCommand::Close
        | HostCommand::Reopen
        | HostCommand::ProgramTransition { .. }
        | HostCommand::ResumeTransition => None,
    }
}
pub(super) fn checkpoint(
    home: &OracleHome,
    kind: CheckpointKind,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    match kind {
        CheckpointKind::Explicit => {
            home.checkpoint(execution)
                .unwrap_or_else(|cause| panic!("actual current checkpoint: {cause:?}"));
        }
        CheckpointKind::Automatic => {
            // The actual opening policy schedules maintenance after mutations.
            let observation = home.inspect_checkpoint(execution).unwrap();
            assert!(
                observation.journals.iter().any(|j| j.covered > 0),
                "automatic maintenance acknowledged a real journal prefix"
            );
            assert!(
                !observation.due,
                "automatic maintenance left an overdue prefix: {observation:?}"
            );
        }
        CheckpointKind::ImageAdmission => {
            unreachable!("inactive owner-gated command is never dispatched")
        }
    }
}
pub(super) fn limits() -> worth_store_application::ApplicationHomeCheckpointLimits {
    worth_store_application::ApplicationHomeCheckpointLimits::new(
        std::num::NonZeroU64::new(8).unwrap(),
        std::num::NonZeroU64::new(64 << 20).unwrap(),
    )
}
