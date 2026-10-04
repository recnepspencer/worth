use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use std::sync::Arc;

use super::supersession::reject_older_successor;
use super::{
    supersede_predecessors, DemandAdmissionKind, DemandRecord, DemandState,
    OutputRefreshPredecessor, WorthQueryOutputDemandInterest, WorthQueryOutputDemandKey,
    WorthQueryOutputDemandNotifications, WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

mod obligation_capacity;
mod performed;
pub(super) use obligation_capacity::performed_obligation_capacity_denial;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        selected_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        admission_kind: DemandAdmissionKind,
        expected_source_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
        successor_of: Option<OutputRefreshPredecessor<'_>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandInterest, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let stable_refresh = matches!(successor_of, Some(OutputRefreshPredecessor::Stable { .. }));
        let matching_delivery = if stable_refresh {
            None
        } else {
            state
                .records
                .iter()
                .find(|(key, record)| {
                    let pending_commit = match &record.state {
                        DemandState::Output(output) => output.published_commit.as_ref(),
                        _ => None,
                    };
                    key.same_occurrence(&requested_key)
                        && selected_commit.is_some_and(|selected| pending_commit == Some(selected))
                })
                .map(|(key, _)| key.clone())
        };
        let matching_semantic_source = if stable_refresh {
            None
        } else {
            newest_semantic_key(&state, &requested_key, accepts_semantic_join)
        };
        let requested_source = requested_key.source.clone();
        let mut retained_key = None;
        if admission_kind == DemandAdmissionKind::Recovery {
            let expected = expected_source_commit.ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "recovery requires the exact source publication",
                )
            })?;
            let custody = state.source_custody.get(expected).ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "the requested publication has no retained output custody",
                )
            })?;
            if let Some(retired) = &custody.retired {
                return Err(retired.clone());
            }
            if let Some(retired) = custody.source_denial(&requested_source) {
                return Err(retired);
            }
            let same_occurrence = custody.occurrence == product_occurrence;
            retained_key = newest_semantic_key(&state, &requested_key, |record| {
                (record.is_required()
                    || record.has_cached_ready()
                    || record.performed_source.as_ref().is_some_and(|source| {
                        source.change.product_commit() == expected
                            && source.output_source_identity.as_ref() == Some(&requested_source)
                    }))
                    && record.source_commits.contains(expected)
                    && record.product_occurrence == product_occurrence
                    && record.source_scope == Some(source_scope)
            });
            if let Some(key) = retained_key.as_ref() {
                state.charge_record_lookup(key, admission)?;
            }
            if let Some(DemandState::Failed(denial)) = retained_key
                .as_ref()
                .and_then(|key| state.records.get(key))
                .map(|record| &record.state)
            {
                return Err(denial.clone());
            }
            let retained_record = retained_key.is_some();
            let available = custody.available(&requested_source, source_scope);
            if !same_occurrence || (!retained_record && !available) {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "exact publication, source scope, and root identity are not retained",
                ));
            }
        }
        state.charge_record_lookup(&requested_key, admission)?;
        let existing_key = if stable_refresh {
            // A refreshed Stable alias is bound to the freshly selected
            // source epoch. A semantic join with an older row would retain
            // the predecessor's obsolete source address.
            state
                .records
                .contains_key(&requested_key)
                .then(|| requested_key.clone())
        } else if retained_key.is_some() {
            retained_key
        } else if state.records.contains_key(&requested_key) {
            Some(requested_key.clone())
        } else if let Some(key) = matching_semantic_source {
            Some(key)
        } else if let Some(key) = matching_delivery {
            Some(key)
        } else {
            matching_delivery
        };
        let existing_record = existing_key.is_some();
        let key = existing_key
            .as_ref()
            .cloned()
            .unwrap_or_else(|| requested_key.clone());
        if let Some(predecessor) = successor_of {
            predecessor.validate_stable_interest(
                self,
                &state,
                &key,
                source_scope,
                product_occurrence,
            )?;
        }
        admission.charge_external_work(4).map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "required work activation exceeds request work",
            )
        })?;
        let required_member = state.prepare_required_member(&key)?;
        if let Some(successor) = successor_of {
            state.charge_record_lookup(&key, admission)?;
            if let Some(DemandState::Output(output)) =
                state.records.get(&key).map(|record| &record.state)
            {
                let matches_ready = output.checkpoint.as_ref().is_some_and(|checkpoint| {
                    matches!(checkpoint, super::WorthQueryOutputCheckpoint::Ready(completion)
                        if successor.matches_ready(&completion.authority))
                });
                if matches_ready {
                    if let super::WorthQueryOutputAdvancement::Stopped { denial, .. } =
                        &output.advancement
                    {
                        return Err(denial.clone());
                    }
                }
            }
        }
        state.charge_record_lookup(&key, admission)?;
        let accepts_prepared_source = state.records.get(&key).is_none_or(|record| {
            record.performed_source.is_none() && matches!(record.state, DemandState::Admitted)
        });
        let prepared_commit = accepts_prepared_source
            .then(|| match admission_kind {
                DemandAdmissionKind::Recovery => expected_source_commit.and_then(|expected| {
                    state
                        .source_custody
                        .get(expected)
                        .filter(|custody| custody.available(&requested_source, source_scope))
                        .map(|_| expected.clone())
                }),
                DemandAdmissionKind::Required => selected_commit.and_then(|selected| {
                    state
                        .source_custody
                        .get(selected)
                        .filter(|custody| custody.available(&requested_source, source_scope))
                        .map(|_| selected.clone())
                }),
                DemandAdmissionKind::Ordinary => None,
            })
            .flatten();
        let prepared_source_commit = prepared_commit.clone();
        let prepared_record = if existing_record {
            None
        } else {
            Some(state.prepare_new_record(
                &key,
                prepared_source_commit.as_ref(),
                product_occurrence,
                source_scope,
                successor_of.map(OutputRefreshPredecessor::key_identity),
                admission,
            )?)
        };
        if prepared_source_commit.is_some() {
            state.charge_record_lookup(&key, admission)?;
        }
        let prepared_commit_growth = if let (Some(commit), Some(record)) =
            (prepared_source_commit.as_ref(), state.records.get(&key))
        {
            if record.source_commits.contains(commit) {
                None
            } else {
                state.prepare_source_commit_growth(record, admission)?
            }
        } else {
            None
        };
        if existing_record {
            state.charge_record_lookup(&key, admission)?;
        } else {
            state.charge_record_lookup_after_insert(&key, admission)?;
        }
        if existing_key.is_none() {
            if successor_of.is_some() {
                // Refresh transfers any performed obligation only after the
                // replacement is admitted and its custody is prepared.
                reject_older_successor(&state, &requested_key)?;
            } else {
                supersede_predecessors(&mut state, &requested_key)?;
            }
        }
        let prepared_source = prepared_commit.and_then(|commit| {
            let custody = state
                .source_custody
                .get_mut(&commit)
                .expect("selected custody exists");
            let performed = custody.source.as_ref().map(|source| {
                let mut source = source.clone();
                source.output_source_identity = Some(requested_source.clone());
                source
            });
            custody.finish_admission(requested_source);
            performed
        });
        if let Some(prepared) = prepared_record {
            state.install_prepared_record(prepared);
        }
        let record = state.records.get_mut(&key).expect("admitted record exists");
        if let Some(successor) = successor_of {
            let reopens_exact_ready = matches!(
                &record.state,
                DemandState::Output(output)
                    if matches!(output.advancement, super::WorthQueryOutputAdvancement::Idle)
                    && output.checkpoint.as_ref().is_some_and(|checkpoint| {
                        matches!(checkpoint, super::WorthQueryOutputCheckpoint::Ready(completion)
                            if successor.matches_ready(&completion.authority))
                    })
            );
            if existing_record && reopens_exact_ready {
                let DemandState::Output(reopened) =
                    std::mem::replace(&mut record.state, DemandState::Admitted)
                else {
                    unreachable!("the reopened row was checked Ready above");
                };
                record.performed_source = None;
                record.successor_of = Some(super::succession::Succession::reopening(
                    successor.key_identity(),
                    reopened,
                ));
                record.wake.notify();
            }
        }
        if record.performed_source.is_none() {
            record.performed_source = prepared_source;
        }
        if let Some(commit) = prepared_source_commit {
            if !record.source_commits.contains(&commit) {
                if let Some(growth) = prepared_commit_growth {
                    growth.install(record);
                }
                record.source_commits.push(commit);
            }
        }
        record.interests = record.interests.saturating_add(1);
        record.required_interests += usize::from(admission_kind.is_required());
        let interest = interest(self, key, record, admission_kind.is_required());
        state.install_required_member(required_member);
        Ok(interest)
    }
}

pub(super) fn accepts_semantic_join(record: &DemandRecord) -> bool {
    match &record.state {
        DemandState::Failed(_) => false,
        DemandState::Output(output) => !matches!(
            output.advancement,
            super::WorthQueryOutputAdvancement::Stopped { .. }
        ),
        _ => true,
    }
}

pub(super) fn newest_semantic_key(
    state: &super::DemandRegistryState,
    requested: &WorthQueryOutputDemandKey,
    accepts: impl Fn(&DemandRecord) -> bool,
) -> Option<WorthQueryOutputDemandKey> {
    state
        .records
        .iter()
        .filter(|(key, record)| key.same_semantic_source(requested) && accepts(record))
        .max_by_key(|(key, _)| key.source.observation_generation())
        .map(|(key, _)| key.clone())
}

pub(super) fn interest(
    owner: &WorthQueryOutputDemandRegistry,
    key: WorthQueryOutputDemandKey,
    record: &DemandRecord,
    requires_output: bool,
) -> WorthQueryOutputDemandInterest {
    WorthQueryOutputDemandInterest {
        key,
        requires_output,
        notifications: WorthQueryOutputDemandNotifications {
            wake: Arc::clone(&record.wake),
        },
        owner: owner.clone(),
    }
}
