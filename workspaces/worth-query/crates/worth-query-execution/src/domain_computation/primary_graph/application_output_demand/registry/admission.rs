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
                    DemandState::DeliveryPending(Some(pending)) => Some(
                        pending
                            .receipt
                            .committed_product_publication()
                            .composite_commit(),
                    ),
                    DemandState::ReadinessPending(Some(pending)) => Some(
                        pending
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
        let requested_source = requested_key.source;
        if admission_kind == DemandAdmissionKind::Recovery {
            let retained_record = state
                .records
                .get(&requested_key)
                .is_some_and(|record| record.required)
                || matching_delivery
                    .as_ref()
                    .and_then(|key| state.records.get(key))
                    .is_some_and(|record| record.required);
            let stale_source = state.prepared_sources.iter().position(|(_, source)| {
                source.receipt.product_branch().occurrence() == product_occurrence
                    && source.receipt.principal_scope().scope() == source_scope
                    && source.current_source_identity.is_some_and(|bound| {
                        WorthQueryOutputDemandKey::source_same_occurrence(&bound, &requested_source)
                            && WorthQueryOutputDemandKey::source_revision(&bound)
                                < WorthQueryOutputDemandKey::source_revision(&requested_source)
                    })
            });
            if let Some(stale_source) = stale_source {
                let (commit, _) = state.prepared_sources.swap_remove(stale_source);
                state.retired_prepared_sources.insert(
                    commit,
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "a newer output source revision retired this prepared custody",
                    ),
                );
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "prepared output source was superseded by a newer source revision",
                ));
            }
            let retained_source = state.prepared_sources.iter().any(|(_, source)| {
                source.receipt.product_branch().occurrence() == product_occurrence
                    && source.receipt.principal_scope().scope() == source_scope
                    && source.current_source_identity == Some(requested_source)
            });
            if !retained_record && !retained_source {
                let newer_source_is_retained = state.prepared_sources.iter().any(|(_, source)| {
                    source.receipt.product_branch().occurrence() == product_occurrence
                        && source.receipt.principal_scope().scope() == source_scope
                        && source.current_source_identity.is_some_and(|bound| {
                            WorthQueryOutputDemandKey::source_same_occurrence(
                                &bound,
                                &requested_source,
                            ) && WorthQueryOutputDemandKey::source_revision(&bound)
                                > WorthQueryOutputDemandKey::source_revision(&requested_source)
                        })
                });
                return Err(WorthQueryOutputDemandDenial::new(
                    if newer_source_is_retained {
                        WorthQueryOutputDemandDenialKind::Superseded
                    } else {
                        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
                    },
                    if newer_source_is_retained {
                        "a newer prepared output source remains in owner custody"
                    } else {
                        "required output has no owner-retained obligation or exact prepared source"
                    },
                ));
            }
        }
        let existing_key = if state.records.contains_key(&requested_key) {
            Some(requested_key.clone())
        } else if let Some(key) = matching_delivery {
            Some(key)
        } else {
            None
        };
        let key = existing_key
            .as_ref()
            .cloned()
            .unwrap_or_else(|| requested_key.clone());
        if let Some(expected) = expected_source_commit {
            let retained_source_commit = state
                .records
                .get(&key)
                .and_then(|record| record.source_commit.as_ref())
                .or_else(|| {
                    state
                        .prepared_sources
                        .iter()
                        .find(|(_, source)| {
                            source.receipt.product_branch().occurrence() == product_occurrence
                                && source.receipt.principal_scope().scope() == source_scope
                                && source.current_source_identity == Some(requested_source)
                        })
                        .map(|(commit, _)| commit)
                });
            if retained_source_commit != Some(expected) {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "recovered output source does not belong to the requested source publication",
                ));
            }
        }
        if existing_key.is_none() {
            supersede_predecessors(&mut state, &requested_key)?;
        }
        let accepts_prepared_source = state.records.get(&key).is_none_or(|record| {
            record.performed_source.is_none() && matches!(record.state, DemandState::Admitted)
        });
        let prepared_source = accepts_prepared_source
            .then(|| {
                let prepared_index = match admission_kind {
                    DemandAdmissionKind::Recovery => {
                        state.prepared_sources.iter().position(|(_, source)| {
                            source.receipt.product_branch().occurrence() == product_occurrence
                                && source.receipt.principal_scope().scope() == source_scope
                                && source.current_source_identity == Some(requested_source)
                        })
                    }
                    DemandAdmissionKind::Required => selected_commit.and_then(|selected| {
                        state.prepared_sources.iter().position(|(commit, source)| {
                            commit == selected
                                && source.receipt.principal_scope().scope() == source_scope
                        })
                    }),
                    DemandAdmissionKind::Ordinary => None,
                };
                prepared_index.map(|index| state.prepared_sources.swap_remove(index).1)
            })
            .flatten();
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
        if record.source_commit.is_none() {
            record.source_commit = prepared_source_commit;
        }
        record.interests = record.interests.saturating_add(1);
        record.required |= admission_kind.is_required();
        Ok(interest(self, key, record))
    }

    pub(in crate::domain_computation::primary_graph) fn admit_performed(
        &self,
        key: WorthQueryOutputDemandKey,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Result<WorthQueryOutputDemandInterest, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(prepared_index) = state.prepared_sources.iter().position(|(commit, source)| {
            commit == source_commit && source.current_source_identity == Some(key.source)
        }) else {
            if let Some(denial) = state.retired_prepared_sources.get(source_commit) {
                return Err(denial.clone());
            }
            if state
                .prepared_sources
                .iter()
                .any(|(commit, _)| commit == source_commit)
            {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "performed source identity does not match its prepared publication",
                ));
            }
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "performed source is absent or has already been consumed",
            ));
        };
        if state
            .records
            .get(&key)
            .is_some_and(|record| record.performed_source_accepted)
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::DuplicatePerformedSource,
                "performed source is already retained by this output demand",
            ));
        }
        supersede_predecessors(&mut state, &key)?;
        let performed_source = state.prepared_sources.swap_remove(prepared_index).1;
        let performed_scope = performed_source.receipt.principal_scope().scope();
        let record = state.records.entry(key.clone()).or_insert_with(|| {
            new_record(
                product_occurrence,
                performed_scope,
                Some(source_commit.clone()),
                true,
            )
        });
        if matches!(record.state, DemandState::Admitted) {
            record.performed_source = Some(performed_source);
        } else {
            drop(performed_source);
        }
        record.performed_source_accepted = true;
        record.interests = record.interests.saturating_add(1);
        Ok(interest(self, key, record))
    }
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
        source_commit,
        state: DemandState::Admitted,
        performed_source: None,
        performed_source_accepted: false,
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
