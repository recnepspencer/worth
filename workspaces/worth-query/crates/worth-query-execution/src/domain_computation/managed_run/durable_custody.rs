//! Home custody follows the actual affinity; native owners alone report release.
use crate::domain_computation::primary_graph::query_obligations::{
    history::QueryObligationHistory, QueryManagedRunPosture, QueryObligationDenial,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
};
use worth_foundational::facade::ManagedRunObligationIdentity;

#[derive(Default)]
pub(in crate::domain_computation) struct ManagedRunCustodyRegistry {
    history: OnceLock<Arc<QueryObligationHistory>>,
    live: Mutex<BTreeMap<String, Arc<ManagedRunCustody>>>,
}
pub(in crate::domain_computation::managed_run) struct ManagedRunIdentity {
    pub(super) logical: Arc<str>,
    pub(super) custody: Option<Arc<ManagedRunCustody>>,
}
pub(in crate::domain_computation) struct ManagedRunCustody {
    pub(super) identity: ManagedRunObligationIdentity,
    recovery: Mutex<Option<crate::domain_computation::primary_graph::query_obligations::QueryManagedRunRecovery>>,
    logical: Arc<str>,
    history: Arc<QueryObligationHistory>,
    posture: Mutex<Option<(QueryManagedRunPosture, String)>>,
    original: Mutex<Option<crate::domain_computation::primary_graph::query_obligations::QueryManagedCleanupOriginal>>,
}
impl ManagedRunCustodyRegistry {
    pub(in crate::domain_computation) fn bind(
        &self,
        history: Arc<QueryObligationHistory>,
    ) -> Result<(), QueryObligationDenial> {
        if let Some(known) = self.history.get() {
            return if Arc::ptr_eq(known, &history) {
                Ok(())
            } else {
                Err(QueryObligationDenial::ForeignHome)
            };
        }
        self.history
            .set(history)
            .map_err(|_| QueryObligationDenial::PublicationPending)
    }
    pub(in crate::domain_computation::managed_run) fn issue(
        &self,
        attempt: &str,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(ManagedRunIdentity, Option<QueryObligationDenial>), QueryObligationDenial> {
        let Some(history) = self.history.get() else {
            return Ok((ManagedRunIdentity::local(), None));
        };
        let known = self
            .live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(attempt)
            .cloned();
        let logical = if let Some(known) = known {
            match known
                .history
                .settle_managed_run_retention(request, known.identity)
            {
                Ok(true) => {
                    return Ok((
                        ManagedRunIdentity {
                            logical: Arc::clone(&known.logical),
                            custody: Some(known),
                        },
                        None,
                    ))
                }
                Err(cause) => {
                    return Ok((
                        ManagedRunIdentity {
                            logical: Arc::clone(&known.logical),
                            custody: Some(known),
                        },
                        Some(cause),
                    ))
                }
                Ok(false) => {
                    self.live
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .remove(attempt);
                    Arc::clone(&known.logical)
                }
            }
        } else {
            super::run_affinity::fresh_logical_identity()
        };
        // Publish prior actual releases before admitting further work. The
        // registry retains active owners and at most the latest completed wave,
        // rather than accumulating terminal observations until process close.
        self.flush(request)?;
        let Some((identity, pending)) = history.retain_managed_run(request, &logical, attempt)?
        else {
            return Ok((
                ManagedRunIdentity {
                    logical,
                    custody: None,
                },
                None,
            ));
        };
        let custody = Arc::new(ManagedRunCustody {
            identity,
            logical: Arc::clone(&logical),
            history: Arc::clone(history),
            posture: Mutex::new(None),
            original: Mutex::new(None),
            recovery: Mutex::new(None),
        });
        self.live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(attempt.to_owned(), Arc::clone(&custody));
        Ok((
            ManagedRunIdentity {
                logical,
                custody: Some(custody),
            },
            pending,
        ))
    }
    pub(in crate::domain_computation) fn flush(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        let live = self
            .live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|(attempt, owner)| (attempt.clone(), Arc::clone(owner)))
            .collect::<Vec<_>>();
        for (attempt, owner) in live {
            let retained = owner
                .history
                .settle_managed_run_retention(request, owner.identity)?;
            let posture = owner
                .posture
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if !retained {
                if posture
                    .as_ref()
                    .is_some_and(|(posture, _)| posture.resources_released())
                {
                    self.live
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .remove(&attempt);
                    continue;
                }
                return Err(QueryObligationDenial::ManagedRunPending {
                    identity: owner.identity,
                });
            }
            let original = *owner
                .original
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(original) = original {
                owner
                    .history
                    .associate_managed_run_original(request, owner.identity, original)?;
            }
            let recovery = *owner
                .recovery
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(cause) = recovery {
                owner
                    .history
                    .associate_managed_run_recovery(request, owner.identity, cause)?;
            }
            if let Some((posture, current_attempt)) = posture {
                owner.history.record_managed_run_handoff(
                    request,
                    owner.identity,
                    posture,
                    &current_attempt,
                )?;
                if posture.resources_released() {
                    self.live
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .remove(&attempt);
                }
            }
        }
        Ok(())
    }
}
impl ManagedRunIdentity {
    pub(super) fn local() -> Self {
        Self {
            logical: super::run_affinity::fresh_logical_identity(),
            custody: None,
        }
    }
}
impl ManagedRunCustody {
    pub(super) fn record_recovery(
        &self,
        cause: crate::domain_computation::primary_graph::query_obligations::QueryManagedRunRecovery,
    ) {
        self.recovery
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_or_insert(cause);
    }
    pub(in crate::domain_computation) fn associate_original(
        &self,
        original: Option<crate::domain_computation::primary_graph::query_obligations::QueryManagedCleanupOriginal>,
    ) {
        if let Some(original) = original {
            let mut known = self
                .original
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(known) = *known {
                assert_eq!(
                    known, original,
                    "the original managed work cannot change World subject"
                );
            }
            *known = Some(original);
        }
    }
    pub(in crate::domain_computation) fn record_admission_release(&self, attempt: &str) {
        self.record(QueryManagedRunPosture::AdmissionReleased, attempt);
    }
    pub(super) fn record(&self, posture: QueryManagedRunPosture, attempt: &str) {
        *self
            .posture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some((posture, attempt.to_owned()));
    }
}
