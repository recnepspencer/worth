use super::{RelationalAttemptProgressPosture, SignalAttemptProgressPosture};

pub(crate) const fn owner_effect_count_from_postures(
    relational: RelationalAttemptProgressPosture,
    signal: SignalAttemptProgressPosture,
) -> usize {
    let relational = match relational {
        RelationalAttemptProgressPosture::Untouched
        | RelationalAttemptProgressPosture::Prepared => 0,
        RelationalAttemptProgressPosture::Performed
        | RelationalAttemptProgressPosture::SettlementRequired
        | RelationalAttemptProgressPosture::SettlementPending
        | RelationalAttemptProgressPosture::Settled => 1,
    };
    let signal = match signal {
        SignalAttemptProgressPosture::Untouched
        | SignalAttemptProgressPosture::PreparedForExecution => 0,
        SignalAttemptProgressPosture::Performed => 1,
    };
    relational + signal
}
