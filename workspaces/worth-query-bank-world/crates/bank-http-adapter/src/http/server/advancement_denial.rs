use crate::{BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next};

// Reuse the existing execution projection; this slice adds custody, not new
// HTTP outcome categories.
pub(super) fn advancement(
    denial: worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial as Denial;
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption as Interruption;
    match denial {
        Denial::Resource(cause) => super::mutation_application::execution_resource(cause).1,
        Denial::Interrupted(Interruption::Cancelled) => {
            BankHttpDenial::new(Kind::Cancelled, Next::Retry)
        }
        Denial::Interrupted(Interruption::DeadlineExceeded) => {
            BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
        }
        Denial::ForeignPhase => BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
        Denial::NestedOpening => BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator),
        Denial::NestedStopped | Denial::Panicked => {
            BankHttpDenial::new(Kind::InternalDenied, Next::None)
        }
    }
}

#[cfg(test)]
mod tests;
