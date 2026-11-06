//! Shared script dispatch runs actual production operations on each Host.
use super::protocol::Disposition;
use super::*;
use host_observation::{
    projection::HeadProjection,
    receipts::{self, MutationProjection},
    requests, OracleHome,
};
use std::collections::HashMap;
enum MutationMode {
    Ordinary,
    LostResponse,
    SettlementFailure,
    Abandonment,
    Concurrent,
}

pub(super) fn mutation(
    home: &OracleHome,
    command: &HostCommand,
    before: &mut HashMap<u64, HeadProjection>,
) -> Option<MutationProjection> {
    let (key, value, mode) = match command {
        HostCommand::Mutate { key, value, .. }
        | HostCommand::MoveProduct { key, value, .. }
        | HostCommand::Retry { key, value } => (*key, *value, MutationMode::Ordinary),
        HostCommand::LoseResponse { key, value } => (*key, *value, MutationMode::LostResponse),
        HostCommand::SettlementFailure { key, value } => {
            (*key, *value, MutationMode::SettlementFailure)
        }
        HostCommand::AbandonCaller { key, value } => (*key, *value, MutationMode::Abandonment),
        HostCommand::ConcurrentSameKey { key, value } => (*key, *value, MutationMode::Concurrent),
        HostCommand::Open
        | HostCommand::Initialize
        | HostCommand::CreateBranch(_)
        | HostCommand::ContinueCustody { .. }
        | HostCommand::ReleaseCustody
        | HostCommand::Checkpoint(_)
        | HostCommand::Close
        | HostCommand::Reopen
        | HostCommand::ProgramTransition { .. }
        | HostCommand::ResumeTransition => return None,
    };
    let head =
        HeadProjection::observe(&home.runtime().runtime().application_oracle_head_for_test());
    let original = before.get(&key).unwrap_or(&head).clone();
    let outcome = match mode {
        MutationMode::SettlementFailure => {
            host_interruption::publication::settlement_failure(home.runtime(), key, value)
        }
        MutationMode::Abandonment => {
            host_interruption::publication::abandon_after_owner(home.runtime(), key, value)
        }
        MutationMode::Concurrent => {
            host_observation::concurrent_mutation::execute(home.runtime(), key, value, &original)
        }
        MutationMode::Ordinary | MutationMode::LostResponse => {
            requests::set(home.runtime(), key, value)
        }
    };
    let observed = receipts::observe(&outcome, &original);
    if observed.fate == receipts::MutationFate::Committed {
        before.entry(key).or_insert(head);
    }
    // The observer may see state; the application response is deliberately lost.
    if matches!(mode, MutationMode::LostResponse) {
        None
    } else {
        Some(observed)
    }
}
pub(super) fn disposition(home: &OracleHome, command: &HostCommand) -> Option<Disposition> {
    match command {
        HostCommand::ContinueCustody { key, value } => {
            use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution;
            let resolved =
                host_interruption::publication::continue_custody(home.runtime(), *key, *value)
                    .unwrap_or_else(|denial| {
                        panic!("original Query settlement continuation: {denial:?}")
                    });
            match resolved {
                Resolution::RecoveredCommitted(receipt) => Some(Disposition::Continued(
                    host_observation::receipts::observe_original(&receipt),
                )),
                Resolution::ProductUnpublished(effects) => {
                    panic!("original remains unpublished: {effects:?}")
                }
                Resolution::ProductPreparationPending(pending) => {
                    panic!("original remains pending: {pending:?}")
                }
                Resolution::ReleasedOriginal(original) => {
                    panic!("released original cannot be continued: {original:?}")
                }
                Resolution::Unseen => panic!("original settlement key is absent"),
                Resolution::IntentDrift => panic!("original settlement intent changed"),
            }
        }
        HostCommand::ReleaseCustody => Some(Disposition::Released(
            host_interruption::publication::release_custody(home.runtime()).unwrap(),
        )),
        HostCommand::Open
        | HostCommand::Initialize
        | HostCommand::Mutate { .. }
        | HostCommand::LoseResponse { .. }
        | HostCommand::Retry { .. }
        | HostCommand::ConcurrentSameKey { .. }
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
            // This entire command stays inactive until P-A FUNDING LANDED.
            // The independent bracket proves a particular ordinary mutation
            // caused count crossing and acknowledgement without a manual call.
            super::automatic::run();
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
