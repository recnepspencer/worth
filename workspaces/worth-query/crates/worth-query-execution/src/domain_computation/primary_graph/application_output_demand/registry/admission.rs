use std::sync::{Arc, Condvar, Mutex};

use super::{
    supersede_predecessors, DemandAdmissionKind, DemandRecord, DemandState, DemandWake,
    WorthQueryOutputDemandInterest, WorthQueryOutputDemandKey, WorthQueryOutputDemandNotifications,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        selected_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        admission_kind: DemandAdmissionKind,
        expected_source_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
    ) -> Result<WorthQueryOutputDemandInterest, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let matching_delivery = state
            .records
            .iter()
            .find(|(key, record)| {
                let pending_commit = match &record.state {
                    DemandState::Output(output) => Some(
                        output
                            .receipt
                            .committed_product_publication()
                            .composite_commit(),
                    ),
                    _ => None,
                };
                key.same_occurrence(&requested_key)
                    && selected_commit.is_some_and(|selected| pending_commit == Some(selected))
            })
            .map(|(key, _)| key.clone());
        let matching_semantic_source = newest_semantic_key(&state, &requested_key, |record| {
            accepts_semantic_join(record)
        });
        let requested_source = requested_key.source;
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
                record.required
                    && record.source_commits.contains(expected)
                    && record.product_occurrence == product_occurrence
                    && record.source_scope == Some(source_scope)
            });
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
        let existing_key = if retained_key.is_some() {
            retained_key
        } else if state.records.contains_key(&requested_key) {
            Some(requested_key.clone())
        } else if let Some(key) = matching_semantic_source {
            Some(key)
        } else if let Some(key) = matching_delivery {
            Some(key)
        } else {
            None
        };
        let key = existing_key
            .as_ref()
            .cloned()
            .unwrap_or_else(|| requested_key.clone());
        if existing_key.is_none() {
            supersede_predecessors(&mut state, &requested_key)?;
        }
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
        let prepared_source = prepared_commit.and_then(|commit| {
            let custody = state
                .source_custody
                .get_mut(&commit)
                .expect("selected custody exists");
            let performed = custody.source.as_ref().map(|source| {
                let mut source = source.clone();
                source.output_source_identity = Some(requested_source);
                source
            });
            custody.finish_admission(requested_source);
            performed
        });
        let prepared_source_commit = prepared_source
            .as_ref()
            .map(|source| source.change.product_commit().clone());
        let record = state.records.entry(key.clone()).or_insert_with(|| {
            new_record(
                product_occurrence,
                source_scope,
                prepared_source_commit.clone(),
                admission_kind.is_required(),
            )
        });
        if record.performed_source.is_none() {
            record.performed_source = prepared_source;
        }
        if let Some(commit) = prepared_source_commit {
            if !record.source_commits.contains(&commit) {
                record.source_commits.push(commit);
            }
        }
        record.interests = record.interests.saturating_add(1);
        record.required |= admission_kind.is_required();
        Ok(interest(self, key, record))
    }

    pub(in crate::domain_computation::primary_graph) fn admit_performed(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Result<WorthQueryOutputDemandInterest, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(custody) = state.source_custody.get(source_commit) else {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "performed source is absent or has already been consumed",
            ));
        };
        if let Some(denial) = &custody.retired {
            return Err(denial.clone());
        }
        if let Some(denial) = custody.source_denial(&requested_key.source) {
            return Err(denial);
        }
        if !custody.available(&requested_key.source, source_scope) {
            if custody.bound_sources.is_some() {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "performed source identity does not match its prepared publication",
                ));
            }
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "performed source is absent or has already been consumed",
            ));
        }
        let key = newest_semantic_key(&state, &requested_key, accepts_semantic_join)
            .unwrap_or_else(|| requested_key.clone());
        if let Some(record) = state.records.get(&key) {
            if record.product_occurrence != product_occurrence
                || record.source_scope != Some(source_scope)
            {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "matching output identity belongs to another source scope or occurrence",
                ));
            }
            if let DemandState::Failed(denial) = &record.state {
                return Err(denial.clone());
            }
        }
        supersede_predecessors(&mut state, &requested_key)?;
        let custody = state
            .source_custody
            .get_mut(source_commit)
            .expect("selected custody exists");
        let mut performed_source = custody
            .source
            .as_ref()
            .expect("available custody retains source")
            .clone();
        performed_source.output_source_identity = Some(requested_key.source);
        custody.finish_admission(requested_key.source);
        let record = state.records.entry(key.clone()).or_insert_with(|| {
            new_record(
                product_occurrence,
                source_scope,
                Some(source_commit.clone()),
                true,
            )
        });
        if !record.source_commits.contains(source_commit) {
            record.source_commits.push(source_commit.clone());
        }
        record.required = true;
        if matches!(record.state, DemandState::Admitted) && record.performed_source.is_none() {
            record.performed_source = Some(performed_source);
        } else {
            drop(performed_source);
        }
        record.interests = record.interests.saturating_add(1);
        Ok(interest(self, key, record))
    }
}

fn accepts_semantic_join(record: &DemandRecord) -> bool {
    match &record.state {
        DemandState::Failed(_) => false,
        DemandState::Output(output) => !matches!(
            output.advancement,
            super::WorthQueryOutputAdvancement::Stopped { .. }
        ),
        _ => true,
    }
}

fn newest_semantic_key(
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

fn new_record(
    product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    source_commit: Option<worth_runtime_world::facade::CompositeCommitIdentity>,
    required: bool,
) -> DemandRecord {
    DemandRecord {
        interests: 0,
        required,
        product_occurrence,
        source_scope: Some(source_scope),
        source_commits: source_commit.into_iter().collect(),
        state: DemandState::Admitted,
        performed_source: None,
        successor_of: None,
        wake: Arc::new(DemandWake {
            generation: Mutex::new(0),
            changed: Condvar::new(),
        }),
    }
}

fn interest(
    owner: &WorthQueryOutputDemandRegistry,
    key: WorthQueryOutputDemandKey,
    record: &DemandRecord,
) -> WorthQueryOutputDemandInterest {
    WorthQueryOutputDemandInterest {
        key,
        notifications: WorthQueryOutputDemandNotifications {
            wake: Arc::clone(&record.wake),
        },
        owner: owner.clone(),
    }
}
