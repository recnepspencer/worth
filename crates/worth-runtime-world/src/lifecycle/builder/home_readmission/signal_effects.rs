//! Join every actual Signal association before constructing a serving World owner.
use crate::durability::{
    WorldDurableHomeBinding, WorldOpenComponentIntent, WorldReadmissionDenial as D,
    WorldRecoveredJournalState,
};
use std::ops::ControlFlow;
use worth_foundational::facade::{
    CanonicalDigestId, DurableOwnerFamily, UncleanRecoveryUnsupported,
};
use worth_signal::facade::durability::SignalDurableBranchSourcePort;

pub(super) fn readmit<Dm, I, T>(
    signal: &SignalDurableBranchSourcePort<Dm, I, T>,
    recovered: &WorldRecoveredJournalState,
    binding: &WorldDurableHomeBinding,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), D>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    let result = signal
        .inspect_associated_advances(request, |effect| {
            let association = effect.association().expect("associated original visitor");
            let unclean = || {
                D::UncleanRecoveryUnsupported(UncleanRecoveryUnsupported {
                    owner: DurableOwnerFamily::Signal,
                    effect: CanonicalDigestId::new(effect.digest()),
                })
            };
            if association.attempt().owner() != binding.owner() {
                return ControlFlow::Break(D::ForeignOwner);
            }
            let Some(open) = recovered.opens.get(&association.attempt()) else {
                return ControlFlow::Break(unclean());
            };
            if &open.association != association {
                return ControlFlow::Break(D::ConflictingOpenAttempt);
            }
            match open.component_intent {
                WorldOpenComponentIntent::RelationalOnly => {
                    return ControlFlow::Break(D::OccurrenceEvidenceMismatch)
                }
                WorldOpenComponentIntent::SignalOnly
                | WorldOpenComponentIntent::RelationalAndSignal => {}
            }
            let occurrence = recovered
                .occurrences
                .values()
                .find(|row| &row.open().association == association);
            match occurrence {
                Some(row) => {
                    if !row
                        .owners()
                        .signal()
                        .is_some_and(|signal| signal.point().reference() == effect.reference())
                    {
                        return ControlFlow::Break(D::OccurrenceEvidenceMismatch);
                    }
                }
                None if !recovered.dispositions.contains_key(&association.attempt()) => {
                    return ControlFlow::Break(unclean())
                }
                None => {}
            }
            ControlFlow::Continue(())
        })
        .map_err(|cause| D::SignalDiscovery(Box::new(cause)))?;
    match result {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(cause) => Err(cause),
    }
}
