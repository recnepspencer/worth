//! A demand owns acknowledged membership independently of optional interests.
use super::*;
use crate::domain_computation::primary_graph::query_obligations::{
    history::QueryObligationHistory, QueryOutputDemandObligation,
};
use std::num::NonZeroU64;
use worth_foundational::facade::DurableCompositeCommitIdentity;

pub struct DurableDemandMembership {
    pub history: Arc<QueryObligationHistory>,
    pub position: NonZeroU64,
    pub completed: std::sync::atomic::AtomicBool,
}
impl DurableDemandMembership {
    pub fn pending(&self) -> bool {
        !self.completed.load(std::sync::atomic::Ordering::Acquire)
    }
    pub fn completed(&self) {
        if !self
            .completed
            .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            self.history.note_output_demand_completion(self.position);
        }
    }
}
/// Ordinary interest shares the already retained query data. Encoding happens
/// only when a performed source or a consumed edge makes it mandatory.
pub struct DurableDemandRecording {
    pub history: Arc<QueryObligationHistory>,
    pub revision: worth_foundational::facade::CanonicalDigestId,
    pub family: String,
    pub producer: String,
    pub query: [u8; 32],
    pub branch: String,
    pub lifecycle: u64,
    pub basis: std::sync::Mutex<DurableCompositeCommitIdentity>,
    pub root: worth_relational::facade::identity::EntityId,
    pub parameters: Arc<crate::domain_computation::primary_graph::application_query::WorthQueryRetainedObservedParameters>,
    pub scope: Arc<crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector>,
}
impl DurableDemandRecording {
    pub fn basis(&self) -> DurableCompositeCommitIdentity {
        *self
            .basis
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn set_basis(&self, basis: DurableCompositeCommitIdentity) {
        *self
            .basis
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = basis;
    }

    pub fn capture(&self) -> Result<QueryOutputDemandObligation, WorthQueryOutputDemandDenial> {
        let mut root = [0; 16];
        root[..4].copy_from_slice(&self.root.partition_value().to_le_bytes());
        root[4..12].copy_from_slice(&self.root.local_slot_value().to_le_bytes());
        root[12..].copy_from_slice(&self.root.generation_value().to_le_bytes());
        let locator = self.scope.locator();
        let fields = locator
            .field_path()
            .fields()
            .iter()
            .map(|field| field.as_str())
            .collect::<Vec<_>>();
        let encode_denial = |_| {
            WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ObligationPublication(crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial::MalformedFrame), "required query input encoding")
        };
        let parameters =
            rmp_serde::to_vec_named(self.parameters.bindings()).map_err(encode_denial)?;
        let scope = rmp_serde::to_vec_named(&(
            locator.aspect().aspect_key().as_str(),
            fields,
            self.scope.value(),
            self.scope.subject(),
        ))
        .map_err(encode_denial)?;
        Ok(QueryOutputDemandObligation {
            acquire_position: NonZeroU64::MIN,
            revision: self.revision,
            family: self.family.clone(),
            producer: self.producer.clone(),
            query: self.query,
            branch: self.branch.clone(),
            basis: self.basis(),
            root,
            parameters,
            scope,
            performed_sources: Vec::new(),
            prerequisites: Vec::new(),
            checkpoint_prerequisites: Vec::new(),
        })
    }
}
impl RequiredOutputReadmission {
    pub fn retain_durable(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis_override: Option<DurableCompositeCommitIdentity>,
        mut sources: Vec<DurableCompositeCommitIdentity>,
        mut prerequisites: Vec<NonZeroU64>,
        mut checkpoints: Vec<Vec<u8>>,
    ) -> Result<Option<NonZeroU64>, WorthQueryOutputDemandDenial> {
        let Some(recording) = self.recording.get() else {
            return Ok(None);
        };
        let mut membership = self
            .durable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(held) = membership.as_ref().filter(|held| held.pending()) {
            let result = held.history.link_output_demand(
                execution,
                held.position,
                recording.lifecycle,
                sources.clone(),
                prerequisites.clone(),
                checkpoints.clone(),
            );
            match result {
                Ok(()) => return Ok(Some(held.position)),
                Err(cause) => {
                    let absent = !held.history.output_demand_retained(held.position);
                    if absent && matches!(cause,
                        crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial::PublicationNoEffect |
                        crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial::UnknownObligation) {
                        *membership = None;
                    } else {
                        return Err(WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::ObligationPublication(cause), "required output links"));
                    }
                }
            }
        }
        sources.sort_unstable();
        sources.dedup();
        prerequisites.sort_unstable();
        prerequisites.dedup();
        let mut row = recording.capture()?;
        if let Some(basis) = basis_override {
            row.basis = basis;
            recording.set_basis(basis);
        }
        row.performed_sources = sources;
        row.prerequisites = prerequisites;
        checkpoints.sort_unstable();
        checkpoints.dedup();
        row.checkpoint_prerequisites = checkpoints;
        let retained = recording
            .history
            .retain_output_demand(execution, row, recording.lifecycle)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ObligationPublication(cause),
                    "required output retention",
                )
            })?;
        let Some(retained) = retained else {
            return Ok(None);
        };
        let position = retained.position;
        *self
            .recorded_prerequisites
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *membership = Some(DurableDemandMembership {
            history: Arc::clone(&recording.history),
            position,
            completed: std::sync::atomic::AtomicBool::new(false),
        });
        if let Some(cause) = retained.pending_denial {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ObligationPublication(cause),
                "required output retention",
            ));
        }
        Ok(Some(position))
    }
    pub fn rejoin_durable(&self) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(recording) = self.recording.get() else {
            return Ok(());
        };
        if !recording.history.has_output_demands() {
            return Ok(());
        }
        let mut held = self
            .durable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if held.as_ref().is_some_and(|row| row.pending()) {
            return Ok(());
        }
        if let Some(row) = recording
            .history
            .matching_output_demand(&recording.capture()?)
        {
            *self
                .recorded_prerequisites
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                (!row.prerequisites.is_empty()).then_some(row.prerequisites);
            *held = Some(DurableDemandMembership {
                history: Arc::clone(&recording.history),
                position: row.acquire_position,
                completed: std::sync::atomic::AtomicBool::new(false),
            });
        }
        Ok(())
    }
    pub fn adopt_durable(&self, history: Arc<QueryObligationHistory>, position: NonZeroU64) {
        let mut held = self
            .durable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let completed = !history.output_demand_retained(position);
        *self
            .recorded_prerequisites
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = (!completed)
            .then(|| history.output_demand(position).map(|row| row.prerequisites))
            .flatten()
            .filter(|prerequisites| !prerequisites.is_empty());
        *held = Some(DurableDemandMembership {
            history,
            position,
            completed: std::sync::atomic::AtomicBool::new(completed),
        });
    }
    pub fn durable_position(&self) -> Option<NonZeroU64> {
        self.durable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .filter(|held| held.pending())
            .map(|held| held.position)
    }
    pub fn durable_pending(&self) -> bool {
        self.durable_position().is_some()
    }
    pub fn complete_durable(&self) {
        *self
            .recorded_prerequisites
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        if let Some(membership) = self
            .durable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            membership.completed();
        }
    }
}
impl WorthQueryOutputDemandInterest {
    pub fn durable_readmission(&self) -> Option<Arc<RequiredOutputReadmission>> {
        self.owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .records
            .get(&self.key)
            .and_then(|row| row.readmission_source.clone())
    }
    pub fn durable_links(
        &self,
    ) -> (
        Vec<worth_runtime_world::facade::CompositeCommitIdentity>,
        Vec<NonZeroU64>,
    ) {
        let state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(row) = state.records.get(&self.key) else {
            return (Vec::new(), Vec::new());
        };
        let sources = row
            .performed_obligations
            .iter()
            .map(|obligation| obligation.source_commit.clone())
            .collect();
        let prerequisites = row
            .prerequisites
            .iter()
            .filter_map(|key| state.records.get(key))
            .filter_map(|row| row.readmission_source.as_ref()?.durable_position())
            .collect();
        (sources, prerequisites)
    }
}

impl RequiredOutputReadmission {
    pub fn retain_domain_failure(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        denial: &WorthQueryOutputDemandDenial,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let held = self
            .durable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(held) = held.as_ref().filter(|held| held.pending()) else {
            return Ok(());
        };
        denial
            .record_domain_failure()
            .and_then(|failure| {
                held.history
                    .retain_output_domain_failure(execution, held.position, failure)
            })
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ObligationPublication(cause),
                    "terminal producer decision",
                )
            })
    }
}
impl WorthQueryOutputDemandInterest {
    pub fn readmit_domain_failure(&self) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(source) = self.durable_readmission() else {
            return Ok(());
        };
        let failure = {
            let held = source
                .durable
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            held.as_ref()
                .and_then(|held| held.history.output_domain_failure(held.position))
        };
        let Some(failure) = failure else {
            return Ok(());
        };
        let denial =
            WorthQueryOutputDemandDenial::readmit_domain_failure(&failure).map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ObligationPublication(cause),
                    "terminal producer decision",
                )
            })?;
        let mut state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(mut record) = state.records.get_mut(self.key()) else {
            return Ok(());
        };
        record.state = super::super::DemandState::Failed(denial);
        record.performed_source = None;
        record.release_obligations();
        record.finish();
        state.defer_terminal_cleanup(&std::sync::Arc::new(self.key().clone()));
        Ok(())
    }
}
