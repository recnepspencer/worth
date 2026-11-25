//! Publish the actual prerequisite closure before the dependent publication.
use super::super::WorthQueryOutputDemandInterest;
use super::super::{source_readmission::RequiredOutputReadmission, DemandRegistryState};
use super::*;
use std::collections::BTreeSet;
use worth_foundational::facade::DurableCompositeCommitIdentity;

struct RequiredSnapshot {
    source: Arc<RequiredOutputReadmission>,
    checkpoints: Vec<
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
    >,
    performed: Vec<DurableCompositeCommitIdentity>,
    prerequisites: Vec<Arc<RequiredOutputReadmission>>,
}
impl PreparedPrerequisiteClaims {
    pub fn record_durable_prerequisites(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        lineage: &std::sync::Mutex<crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage>,
        mut readmit_observation: impl FnMut(
            DurableCompositeCommitIdentity,
            &str,
        ) -> Result<
            worth_runtime_world::facade::ProductBranchObservation,
            WorthQueryOutputDemandDenial,
        >,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let snapshots = {
            let owner = self.context.registry();
            let state = owner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let retained = state
                .records
                .get(self.context.key())
                .and_then(|row| row.readmission_source.as_ref());
            if retained.is_none_or(|source| source.recording.get().is_none()) {
                return Ok(());
            }
            let mut snapshots = Vec::new();
            let mut visited = BTreeSet::new();
            let mut stack = vec![(Arc::new(self.context.key().clone()), false)];
            while let Some((key, expanded)) = stack.pop() {
                let row = state.records.get(key.as_ref()).ok_or_else(closed_denial)?;
                let mut prerequisites = row.prerequisites.clone();
                if key.as_ref() == self.context.key() {
                    prerequisites.extend(self.predecessors.iter().cloned());
                }
                prerequisites.sort_unstable();
                prerequisites.dedup();
                if expanded {
                    let mut captured = snapshot(&state, row, &prerequisites)?;
                    if key.as_ref() == self.context.key()
                        && let Some(claims) = &self.checkpoint_predecessors
                    {
                        captured
                            .checkpoints
                            .extend(claims.identities().iter().cloned());
                    }
                    snapshots.push(captured);
                } else if visited.insert(key.clone()) {
                    stack.push((key, true));
                    stack.extend(prerequisites.into_iter().rev().map(|key| (key, false)));
                }
            }
            snapshots
        };
        for snapshot in snapshots {
            let prerequisites = snapshot
                .prerequisites
                .iter()
                .filter_map(|source| source.durable_position())
                .collect();
            let checkpoints = {
                let lineage = lineage
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                snapshot
                    .checkpoints
                    .iter()
                    .map(|identity| lineage.encode_checkpoint_prerequisite(identity, execution))
                    .collect::<Result<Vec<_>, _>>()?
            };
            let mut _pin_guards = Vec::new();
            if let Some(recording) = snapshot.source.recording.get()
                && let Some(pins) = recording.history.basis_pins.get()
            {
                let mut required = Vec::with_capacity(1 + snapshot.performed.len());
                required.push(recording.basis);
                required.extend(snapshot.performed.iter().copied());
                _pin_guards.reserve(required.len());
                for commit in required {
                    let guard = match pins
                        .try_retain_in_flight(commit, Arc::clone(&recording.history))
                    {
                        Some(guard) => guard,
                        None => {
                            let observation = snapshot
                                .source
                                .retained_program_basis()
                                .and_then(|basis| {
                                    let selected = basis.selected_commit();
                                    (selected.owner_identity().durable() == Some(commit.owner())
                                        && selected.ordinal() == commit.ordinal())
                                    .then(|| basis.product_observation().clone())
                                })
                                .map(Ok)
                                .unwrap_or_else(|| {
                                    readmit_observation(commit, &recording.branch)
                                })?;
                            pins.pin_in_flight(commit, observation, Arc::clone(&recording.history))
                        }
                    };
                    _pin_guards.push(guard);
                }
            }
            snapshot.source.retain_durable(
                execution,
                snapshot.performed,
                prerequisites,
                checkpoints,
            )?;
        }
        Ok(())
    }
}
fn snapshot(
    state: &DemandRegistryState,
    row: &super::super::DemandRecord,
    prerequisites: &[Arc<WorthQueryOutputDemandKey>],
) -> Result<RequiredSnapshot, WorthQueryOutputDemandDenial> {
    let source = row.readmission_source.clone().ok_or_else(closed_denial)?;
    let performed =
        row.performed_obligations
            .iter()
            .filter_map(|obligation| {
                let commit = &obligation.source_commit;
                commit.owner_identity().durable().map(|owner| {
                    DurableCompositeCommitIdentity::from_recorded(owner, commit.ordinal())
                })
            })
            .collect();
    let prerequisites = prerequisites
        .iter()
        .map(|key| {
            state
                .records
                .get(key.as_ref())
                .and_then(|row| row.readmission_source.clone())
                .ok_or_else(closed_denial)
        })
        .collect::<Result<_, _>>()?;
    Ok(RequiredSnapshot {
        checkpoints: row
            .checkpoint_prerequisites
            .as_ref()
            .map(|claims| claims.identities().to_vec())
            .unwrap_or_default(),
        source,
        performed,
        prerequisites,
    })
}

impl WorthQueryOutputDemandInterest {
    pub fn readmit_prerequisite_links(
        &self,
        prerequisites: &[&WorthQueryOutputDemandInterest],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prior = state
            .records
            .get(self.key())
            .ok_or_else(closed_denial)?
            .prerequisites
            .clone();
        let mut keys = prerequisites
            .iter()
            .map(|interest| Arc::new(interest.key().clone()))
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();
        if keys == prior {
            return Ok(());
        }
        if !prior.is_empty() {
            return Err(coverage_denial());
        }
        validate_predecessors(&state, &keys, std::iter::empty())?;
        let members = keys
            .iter()
            .map(|key| state.prepare_required_member(key))
            .collect::<Result<Vec<_>, _>>()?;
        let batch = state.prepare_claim_readthrough(self.key(), &keys)?;
        if let Some(batch) = &batch {
            batch.install();
        }
        for (key, prepared) in keys.iter().zip(members) {
            state
                .records
                .get_mut(key)
                .ok_or_else(closed_denial)?
                .framework_required_count += 1;
            state.install_required_member(prepared);
        }
        let mut row = state
            .records
            .get_mut(self.key())
            .ok_or_else(closed_denial)?;
        row.prerequisites = keys;
        row.claim_batch = batch;
        row.finish();
        Ok(())
    }
}
