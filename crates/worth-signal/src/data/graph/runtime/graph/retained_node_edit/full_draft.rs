//! Concrete preparation of one full-payload retained node edit.
use super::super::NodeArena;
use super::reservation_admission;
use super::{
    map_capacity_denial, map_mutation_denial, publication_peak_charge, NodeEvaluationRoots,
    PreparedRetainedNodeEdit, RetainedNodeEditDenial, RetainedNodeEditPreparation,
    RetainedNodePayload,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Preparation,
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation,
};
use std::sync::Arc;

enum RetainedNodeEditCallback<'payload, 'observer> {
    Edit(
        &'payload mut [RetainedNodePayload],
        &'payload mut Preparation<'observer>,
    ),
    DropOutput,
}

struct ProducedOutputGuard<'edit, 'observer> {
    edit: &'edit mut dyn for<'payload> FnMut(RetainedNodeEditCallback<'payload, 'observer>),
    armed: bool,
}

impl Drop for ProducedOutputGuard<'_, '_> {
    fn drop(&mut self) {
        if self.armed {
            (self.edit)(RetainedNodeEditCallback::DropOutput);
        }
    }
}

pub(super) struct StagedRetainedNodeEdit {
    pub(super) original: NodeEvaluationRoots,
    pub(super) staged: NodeEvaluationRoots,
    pub(super) charge: Charge,
    pub(super) preparation_custody: SignalConditionalRetentionReservation,
}

pub(super) enum StagedRetainedNodeEditResult {
    Ready(StagedRetainedNodeEdit),
    Rejected(RetainedNodeEditDenial),
}

impl StagedRetainedNodeEditResult {
    pub(super) fn with_output<R>(self, output: R) -> RetainedNodeEditPreparation<R> {
        match self {
            Self::Ready(staged) => RetainedNodeEditPreparation::Ready(PreparedRetainedNodeEdit {
                original: staged.original,
                staged: staged.staged,
                output,
                charge: staged.charge,
                _preparation_custody: staged.preparation_custody,
            }),
            Self::Rejected(denial) => RetainedNodeEditPreparation::Rejected { output, denial },
        }
    }
}

impl NodeArena {
    /// The ceiling covers the three installed evaluation representations,
    /// including untouched backing. The ledger reserves selected payload clones
    /// and staged roots before their allocations. New payload allocations made
    /// by the callback and its returned output require caller reservations.
    pub(crate) fn prepare_retained_node_edits_with_work<R>(
        &self,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        indices: &[usize],
        maximum: Option<Charge>,
        work: &mut Preparation,
        edit: impl FnOnce(&mut [RetainedNodePayload], &mut Preparation) -> R,
    ) -> Result<RetainedNodeEditPreparation<R>, RetainedNodeEditDenial> {
        let mut output = None;
        let mut edit = Some(edit);
        let staged =
            self.prepare_retained_node_edits_core(ledger, indices, maximum, work, &mut |step| {
                match step {
                    RetainedNodeEditCallback::Edit(payloads, preparation) => {
                        output = Some(edit.take().expect("one retained node edit")(
                            payloads,
                            preparation,
                        ));
                    }
                    RetainedNodeEditCallback::DropOutput => {
                        output.take();
                    }
                }
            })?;
        Ok(staged.with_output(output.expect("retained node edit produced its output")))
    }

    #[inline(never)]
    fn prepare_retained_node_edits_core(
        &self,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        indices: &[usize],
        maximum: Option<Charge>,
        work: &mut Preparation,
        edit: &mut dyn for<'payload> FnMut(RetainedNodeEditCallback<'payload, '_>),
    ) -> Result<StagedRetainedNodeEditResult, RetainedNodeEditDenial> {
        self.validate_retained_node_selection(indices, work)?;
        work.reserve_visits(indices.len())
            .map_err(RetainedNodeEditDenial::Accounting)?;
        let draft_custody = reservation_admission::reserve_payload_draft::<RetainedNodePayload>(
            self,
            indices.len(),
            ledger,
            work,
        )?;
        let mut payloads = indices
            .iter()
            .map(|&index| RetainedNodePayload {
                hot: self.hot[index]
                    .as_ref()
                    .expect("validated hot payload")
                    .clone(),
                warm: self.warm[index].clone(),
                cold: self.cold[index].clone(),
            })
            .collect::<Vec<_>>();
        edit(RetainedNodeEditCallback::Edit(&mut payloads, work));
        let mut produced = ProducedOutputGuard { edit, armed: true };
        let mut staged = NodeEvaluationRoots {
            hot: self.hot.clone(),
            warm: self.warm.clone(),
            cold: self.cold.clone(),
            custody: self.retained_node_custody.clone(),
        };
        let prepared = (|| {
            let mut charge = Charge::ZERO;
            for (&index, payload) in indices.iter().zip(payloads) {
                let RetainedNodePayload { hot, warm, cold } = payload;
                let before = [
                    staged
                        .hot
                        .prepared_retained_charge()
                        .map_err(map_mutation_denial)?,
                    staged
                        .warm
                        .prepared_retained_charge()
                        .map_err(map_mutation_denial)?,
                    staged
                        .cold
                        .prepared_retained_charge()
                        .map_err(map_mutation_denial)?,
                ];
                let hot = staged
                    .hot
                    .prepare_retained_replacement(index, Some(hot), work)
                    .map_err(|(_, denial)| map_mutation_denial(denial))?;
                let warm = staged
                    .warm
                    .prepare_retained_replacement(index, warm, work)
                    .map_err(|(_, denial)| map_mutation_denial(denial))?;
                let cold = staged
                    .cold
                    .prepare_retained_replacement(index, cold, work)
                    .map_err(|(_, denial)| map_mutation_denial(denial))?;
                let required = publication_peak_charge(
                    before,
                    [
                        hot.required_charge(),
                        warm.required_charge(),
                        cold.required_charge(),
                    ],
                )
                .map_err(RetainedNodeEditDenial::Accounting)?;
                if let Some(maximum) = maximum.filter(|maximum| required > *maximum) {
                    return Err(RetainedNodeEditDenial::CapacityExhausted { maximum, required });
                }
                let custody = reservation_admission::reserve_staged_roots(ledger, required)?;
                // All three destination borrows remain held through admission.
                // No selected lane allocates replacement storage before this
                // ceiling for every publication prefix has been checked.
                let hot_charge = hot
                    .publish(required)
                    .map_err(|(_, d)| map_capacity_denial(d))?;
                let warm_charge = warm
                    .publish(required)
                    .map_err(|(_, d)| map_capacity_denial(d))?;
                let cold_charge = cold
                    .publish(required)
                    .map_err(|(_, d)| map_capacity_denial(d))?;
                staged.custody = Some(custody);
                charge = hot_charge
                    .checked_add(warm_charge)
                    .and_then(|n| n.checked_add(cold_charge))
                    .map_err(RetainedNodeEditDenial::Accounting)?;
            }
            Ok(charge)
        })();
        let charge = match prepared {
            Ok(charge) => charge,
            Err(denial) => {
                produced.armed = false;
                return Ok(StagedRetainedNodeEditResult::Rejected(denial));
            }
        };
        produced.armed = false;
        Ok(StagedRetainedNodeEditResult::Ready(
            StagedRetainedNodeEdit {
                original: NodeEvaluationRoots {
                    hot: self.hot.clone(),
                    warm: self.warm.clone(),
                    cold: self.cold.clone(),
                    custody: self.retained_node_custody.clone(),
                },
                staged,
                charge,
                preparation_custody: draft_custody,
            },
        ))
    }
}
