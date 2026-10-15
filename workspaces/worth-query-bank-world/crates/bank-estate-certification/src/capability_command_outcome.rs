//! One native phase preserves every incomplete command and capture refusal.
use bank_domain::schema::BankSchema;
use bank_server::{BankApplication, BankCommitReceipt};
use worth_query_host::facade::application_entry::*;

type Command<'r, D> = WorthQueryCapabilityCommand<'r, BankSchema, BankApplication, D>;
type Completed<'r, D> = WorthQueryCompletedCapabilityCommand<'r, BankSchema, BankApplication, D>;
type Terminal<'r, D> = WorthQueryTerminalCapabilityCommand<'r, BankSchema, BankApplication, D>;

/// Certification returns original native custody; it never drives a recovery loop.
pub enum EstateCapabilityCommandOutcome<'r, D, Capture> {
    CaptureRefused(Capture),
    Native(
        WorthQueryCapabilityCommandAdvance<Command<'r, D>, Completed<'r, D>, Terminal<'r, D>, D>,
    ),
    CloseRefused {
        completed: Completed<'r, D>,
        denial: WorthQueryCapabilityCommandDenial<D>,
    },
    Closed(BankCommitReceipt),
}

pub(super) fn advance<'r, D, Capture>(
    command: Result<Command<'r, D>, Capture>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
) -> EstateCapabilityCommandOutcome<'r, D, Capture> {
    let command = match command {
        Ok(command) => command,
        Err(capture) => return EstateCapabilityCommandOutcome::CaptureRefused(capture),
    };
    match command.advance(request) {
        WorthQueryCapabilityCommandAdvance::Completed(completed) => {
            match completed.close_with_evidence(request) {
                Ok(closed) => EstateCapabilityCommandOutcome::Closed(
                    BankCommitReceipt::from_closed_capability_command(closed),
                ),
                Err((completed, denial)) => {
                    EstateCapabilityCommandOutcome::CloseRefused { completed, denial }
                }
            }
        }
        native => EstateCapabilityCommandOutcome::Native(native),
    }
}
