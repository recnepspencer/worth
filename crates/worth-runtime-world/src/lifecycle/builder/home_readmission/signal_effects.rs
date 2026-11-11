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

#[derive(Default)]
pub(super) struct ReadmittedSignalEffects {
    points: worth_execution::ExecutionOrderedMap<
        worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
        std::sync::Arc<worth_signal::facade::durability::SignalReadmittedSourcePoint>,
    >,
}
impl ReadmittedSignalEffects {
    pub(super) fn get(
        &self,
        attempt: &worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
    ) -> Option<std::sync::Arc<worth_signal::facade::durability::SignalReadmittedSourcePoint>> {
        self.points.get(attempt).cloned()
    }
}
pub(super) fn readmit<Dm, I, T>(
    signal: &SignalDurableBranchSourcePort<Dm, I, T>,
    recovered: &WorldRecoveredJournalState,
    binding: &WorldDurableHomeBinding,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<ReadmittedSignalEffects, D>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    let mut effects = ReadmittedSignalEffects::default();
    let result = signal
        .inspect_associated_advances(request, |effect, point| {
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
            if let Some(intent) = recovered.creations.get(&association.attempt()) {
                let original = intent.intent();
                if &original.association != association {
                    return ControlFlow::Break(D::ConflictingCreationIntent);
                }
                let crate::branch::SignalBranchCreationPlan::ForkExact { target } = &original.signal else {
                    return ControlFlow::Break(D::CreationEvidenceMismatch);
                };
                match effect.change() {
                    worth_signal::facade::durability::SignalAuthoredBranchChange::Created { name, parent: Some(_) }
                        if name == target.as_str() => {}
                    worth_signal::facade::durability::SignalAuthoredBranchChange::Created { .. }
                    | worth_signal::facade::durability::SignalAuthoredBranchChange::Advanced
                    | worth_signal::facade::durability::SignalAuthoredBranchChange::SnapshotCaptured
                    | worth_signal::facade::durability::SignalAuthoredBranchChange::Restored
                    | worth_signal::facade::durability::SignalAuthoredBranchChange::Retired =>
                        return ControlFlow::Break(D::CreationEvidenceMismatch),
                }
                let Some(selected) = recovered.created.get(&association.attempt()) else {
                    return ControlFlow::Break(unclean());
                };
                if selected.created().components.signal.point().reference() != effect.reference() {
                    return ControlFlow::Break(D::CreationEvidenceMismatch);
                }
                return ControlFlow::Continue(());
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
                None => {
                    let actual=match signal.readmit_source_point(point, request) {
                        Ok(actual) => std::sync::Arc::new(actual),
                        Err(cause) => return ControlFlow::Break(D::SignalSource(Box::new(cause))),
                    };
                    if effects.points.contains_key(&association.attempt()) {
                        return ControlFlow::Break(D::OccurrenceEvidenceMismatch);
                    }
                    if let Err(cause)=effects.points.insert_cloned_native_for_request(
                        &association.attempt(), &actual, *request) {
                        return ControlFlow::Break(D::Medium(crate::durability::WorldDurableMediumDenial::Ordered(cause)));
                    }
                }
            }
            ControlFlow::Continue(())
        })
        .map_err(|cause| D::SignalDiscovery(Box::new(cause)))?;
    match result {
        ControlFlow::Continue(()) => Ok(effects),
        ControlFlow::Break(cause) => Err(cause),
    }
}
