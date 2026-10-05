//! Role-specific selected-node drafts for one atomic graph epoch.
use super::{
    map_capacity_denial, map_clone_work_denial, map_mutation_denial, publication_peak_charge,
    reservation_admission, NodeArena, NodeEvaluationRoots, OperationalNodePayload,
    PreparedRetainedNodeEdit, RetainedNodeEditDenial, RetainedNodeEditPreparation,
    RetainedNodePayload, SelectedNodeDraft, SelectedNodeRole,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    SignalConditionalRetentionLedger,
};
use std::sync::Arc;

impl NodeArena {
    pub(crate) fn prepare_selected_epoch_nodes<R>(
        &self,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        selections: &[(usize, SelectedNodeRole)],
        maximum: Charge,
        work: &mut Work,
        edit: impl FnOnce(&mut [SelectedNodeDraft], &mut Work) -> R,
    ) -> Result<RetainedNodeEditPreparation<R>, RetainedNodeEditDenial> {
        self.validate_epoch_selection(selections, work)?;
        work.reserve_visits(selections.len())
            .map_err(RetainedNodeEditDenial::Accounting)?;
        let draft_custody = reservation_admission::reserve_payload_draft::<SelectedNodeDraft>(
            self,
            selections.len(),
            ledger,
            work,
        )?;
        let mut payloads = selections
            .iter()
            .map(|&(index, role)| match role {
                SelectedNodeRole::ProducerFull => {
                    SelectedNodeDraft::ProducerFull(RetainedNodePayload {
                        hot: self.hot[index]
                            .as_ref()
                            .expect("validated hot payload")
                            .clone(),
                        warm: self.warm[index].clone(),
                        cold: self.cold[index].clone(),
                    })
                }
                SelectedNodeRole::ConsumerOperational => {
                    SelectedNodeDraft::ConsumerOperational(OperationalNodePayload {
                        hot: self.hot[index]
                            .as_ref()
                            .expect("validated hot payload")
                            .clone(),
                        warm: self.warm[index].operational_consumer_draft(),
                    })
                }
            })
            .collect::<Vec<_>>();
        let output = edit(&mut payloads, work);
        let mut staged = NodeEvaluationRoots {
            hot: self.hot.clone(),
            warm: self.warm.clone(),
            cold: self.cold.clone(),
            custody: self.retained_node_custody.clone(),
        };
        let prepared = (|| {
            let mut charge = Charge::ZERO;
            for (&(index, role), payload) in selections.iter().zip(payloads) {
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
                let (hot_payload, warm_payload, cold_payload) = match (role, payload) {
                    (SelectedNodeRole::ProducerFull, SelectedNodeDraft::ProducerFull(full)) => {
                        (full.hot, full.warm, Some(full.cold))
                    }
                    (
                        SelectedNodeRole::ConsumerOperational,
                        SelectedNodeDraft::ConsumerOperational(operational),
                    ) => (operational.hot, operational.warm, None),
                    _ => unreachable!("sealed epoch selection role"),
                };
                let hot = staged
                    .hot
                    .prepare_retained_replacement(index, Some(hot_payload), work)
                    .map_err(|(_, denial)| map_mutation_denial(denial))?;
                let warm = staged
                    .warm
                    .prepare_retained_replacement(index, warm_payload, work)
                    .map_err(|(_, denial)| map_mutation_denial(denial))?;
                let cold = cold_payload
                    .map(|payload| {
                        staged
                            .cold
                            .prepare_retained_replacement(index, payload, work)
                            .map_err(|(_, denial)| map_mutation_denial(denial))
                    })
                    .transpose()?;
                let required = publication_peak_charge(
                    before,
                    [
                        hot.required_charge(),
                        warm.required_charge(),
                        cold.as_ref()
                            .map_or(before[2], |prepared| prepared.required_charge()),
                    ],
                )
                .map_err(RetainedNodeEditDenial::Accounting)?;
                if required > maximum {
                    return Err(RetainedNodeEditDenial::CapacityExhausted { maximum, required });
                }
                let custody = reservation_admission::reserve_staged_roots(ledger, required)?;
                let hot_charge = hot
                    .publish(maximum)
                    .map_err(|(_, d)| map_capacity_denial(d))?;
                let warm_charge = warm
                    .publish(maximum)
                    .map_err(|(_, d)| map_capacity_denial(d))?;
                let cold_charge = cold
                    .map(|prepared| {
                        prepared
                            .publish(maximum)
                            .map_err(|(_, d)| map_capacity_denial(d))
                    })
                    .transpose()?
                    .unwrap_or(before[2]);
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
            Err(denial) => return Ok(RetainedNodeEditPreparation::Rejected { output, denial }),
        };
        Ok(RetainedNodeEditPreparation::Ready(
            PreparedRetainedNodeEdit {
                original: NodeEvaluationRoots {
                    hot: self.hot.clone(),
                    warm: self.warm.clone(),
                    cold: self.cold.clone(),
                    custody: self.retained_node_custody.clone(),
                },
                staged,
                output,
                charge,
                _preparation_custody: draft_custody,
            },
        ))
    }

    fn validate_epoch_selection(
        &self,
        selections: &[(usize, SelectedNodeRole)],
        work: &mut Work,
    ) -> Result<(), RetainedNodeEditDenial> {
        if selections.is_empty() {
            return Err(RetainedNodeEditDenial::EmptyNodeSelection);
        }
        for _ in 0..4 {
            work.reserve_visits(selections.len())
                .map_err(RetainedNodeEditDenial::Accounting)?;
        }
        if selections.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
            return Err(RetainedNodeEditDenial::UnorderedNodeIndices);
        }
        for &(index, role) in selections {
            work.reserve_visits(
                self.hot
                    .lookup_steps()
                    .saturating_add(self.warm.lookup_steps())
                    .saturating_add(self.cold.lookup_steps())
                    .saturating_mul(3),
            )
            .map_err(RetainedNodeEditDenial::Accounting)?;
            self.hot
                .validate_staged_edit(index)
                .map_err(RetainedNodeEditDenial::Lane)?;
            self.warm
                .validate_staged_edit(index)
                .map_err(RetainedNodeEditDenial::Lane)?;
            if role == SelectedNodeRole::ProducerFull {
                self.cold
                    .validate_staged_edit(index)
                    .map_err(RetainedNodeEditDenial::Lane)?;
            }
            self.hot[index]
                .as_ref()
                .ok_or(RetainedNodeEditDenial::MissingHotPayload)?;
            work.reserve_visits(std::mem::size_of::<crate::data::node::NodeHotData>() + 32)
                .map_err(RetainedNodeEditDenial::Accounting)?;
            let mut copies = crate::logic::evaluation::EvaluationWork::Conditional(work);
            self.warm[index]
                .admit_clone_work(&mut copies)
                .map_err(map_clone_work_denial)?;
            if role == SelectedNodeRole::ProducerFull {
                if let Some(cold) = &self.cold[index] {
                    cold.admit_clone_work(&mut copies)
                        .map_err(map_clone_work_denial)?;
                }
            }
        }
        Ok(())
    }
}
