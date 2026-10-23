//! Type-erased domain values retain the actual inverse binding's result and refusal.
use crate::application_entry::WorthQueryApplicationMutationOutcome;
use std::any::Any;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::ApplicationSchema;

/// The inverse binding's actual domain result or refusal.
/// Inspect its binding identity and downcast to the application's declared value type.
pub struct WorthQueryCorrectionValue {
    binding: &'static str,
    value: Box<dyn Any>,
}
impl WorthQueryCorrectionValue {
    pub fn binding(&self) -> &'static str {
        self.binding
    }
    pub fn downcast_ref<Value: 'static>(&self) -> Option<&Value> {
        self.value.downcast_ref()
    }
}
impl std::fmt::Debug for WorthQueryCorrectionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryCorrectionValue")
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}
/// Ordinary mutation outcomes with domain values retained under their inverse binding.
/// Unpublished and unresolved outcomes preserve their native recovery custody.
pub type WorthQueryApplicationCorrectionOutcome =
    WorthQueryApplicationMutationOutcome<WorthQueryCorrectionValue, WorthQueryCorrectionValue>;

pub(super) fn erase<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    outcome: WorthQueryApplicationMutationOutcome<Binding::Denial, Binding::Result>,
) -> WorthQueryApplicationCorrectionOutcome {
    use WorthQueryApplicationMutationOutcome as Outcome;
    match outcome {
        Outcome::Committed { receipt, result } => Outcome::Committed {
            receipt,
            result: WorthQueryCorrectionValue {
                binding: Binding::IDENTITY,
                value: Box::new(result),
            },
        },
        Outcome::DomainDenied(denial) => Outcome::DomainDenied(WorthQueryCorrectionValue {
            binding: Binding::IDENTITY,
            value: Box::new(denial),
        }),
        Outcome::AlreadyCommitted(receipt) => Outcome::AlreadyCommitted(receipt),
        Outcome::PreviouslyCommitted(receipt) => Outcome::PreviouslyCommitted(receipt),
        Outcome::IdempotencyIntentDrift => Outcome::IdempotencyIntentDrift,
        Outcome::Cancelled => Outcome::Cancelled,
        Outcome::DeadlineExceeded => Outcome::DeadlineExceeded,
        Outcome::Commit(custody) => Outcome::Commit(custody),
    }
}
