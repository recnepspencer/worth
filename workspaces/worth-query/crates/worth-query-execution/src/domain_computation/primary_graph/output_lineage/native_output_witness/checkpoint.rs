//! Recover the original output expectations, never expectations from a new head.
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;

use super::*;
use crate::domain_computation::primary_graph::application_attempt::Movement;

mod fact_index;

use fact_index::CheckpointRoleIndex;

impl SealedNativeOutputWitness {
    /// The original output revisions and the decodable producer observations
    /// must both hold at the authentic recovered Native snapshot. Unsupported
    /// fact probes conservatively require ordinary Fresh execution.
    pub(in crate::domain_computation::primary_graph) fn checkpoint_facts_current_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        facts: &(impl FactSequence + ?Sized),
    ) -> Result<bool, super::super::invalidation::SettlementVerificationStop> {
        if !self.unchanged_in(relational, snapshot, execution)? {
            return Ok(false);
        }
        for fact in facts.iter() {
            match fact {
                Fact::SourceEntity { .. }
                | Fact::AspectPresence { .. }
                | Fact::SourceFieldRevision { .. }
                | Fact::Entity { .. }
                | Fact::SourceAdjacencyRevision { .. }
                | Fact::SourceAspectRevision { .. }
                | Fact::IndexedEntitySelection { .. } => {}
                _ => return Ok(false),
            }
            // The fact owner reserves on this carried meter before reading
            // and settles the actual probe debit on success or failure.
            match fact.source_currentness_in(relational, execution, snapshot)? {
                Err(stop @ crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::ManagedIndex(_)) => return Err(super::super::invalidation::SettlementVerificationStop::SourceRead(stop)),
                Err(stop @ crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::OutputObservation(_)) => return Err(super::super::invalidation::SettlementVerificationStop::SourceRead(stop)),
                Ok(movement) if movement.movement() == Movement::Unmoved => {}
                Err(crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::Execution(cause)) => {
                    return Err(super::super::invalidation::SettlementVerificationStop::SourceRead(crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::Execution(cause)));
                }
                Ok(_) | Err(crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::Unavailable | crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) => return Ok(false),
            }
        }
        Ok(true)
    }

    /// Authenticated facts must cover every installed aspect of every exact
    /// output role. Producer-only legacy payloads cannot invent missing proof.
    /// The roles are indexed once by entity and the facts are read in one
    /// pass, so cost grows with the facts plus the roles, not their product.
    pub(in crate::domain_computation::primary_graph) fn from_checkpoint_facts(
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        layout: &WorthQueryPrimaryGraphLayout,
        facts: &(impl FactSequence + ?Sized),
    ) -> Result<Option<Arc<OnceLock<Self>>>, CompanionPreflightStop> {
        Self::checkpoint_expectations(correspondence, layout, facts, None, &mut |_| Ok(()))
            .map(|witness| witness.map(Self::seal_checkpoint_expectations))
    }

    /// Decode original expectations only. Historical currentness must still
    /// prove those exact versions at H and their unchanged restoration at D.
    pub(in crate::domain_computation::primary_graph) fn from_historical_checkpoint_facts(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        layout: &WorthQueryPrimaryGraphLayout,
        facts: &(impl FactSequence + ?Sized),
    ) -> Result<Option<Arc<OnceLock<Self>>>, CompanionPreflightStop> {
        execution
            .run_native_with_checkpoint(|context| {
                Self::checkpoint_expectations(
                    correspondence,
                    layout,
                    facts,
                    Some(execution),
                    &mut |work| {
                        crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                            context, work,
                        )
                        .map_err(|stop| {
                            CompanionPreflightStop::Execution(
                                worth_execution::WorkCeilingDenial::Stopped(stop).into(),
                            )
                        })
                    },
                )
            })
            .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?
            .0
            .map(|witness| witness.map(Self::seal_checkpoint_expectations))
    }

    fn seal_checkpoint_expectations(witness: Self) -> Arc<OnceLock<Self>> {
        Arc::new(OnceLock::from(witness))
    }

    fn checkpoint_expectations(
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        layout: &WorthQueryPrimaryGraphLayout,
        facts: &(impl FactSequence + ?Sized),
        execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        visit: &mut dyn FnMut(u64) -> Result<(), CompanionPreflightStop>,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let role_count = correspondence.native_witness_roles().len();
        if role_count == 0 {
            return Ok(None);
        }
        let mut aspect_count = 0usize;
        let mut names_bytes = 0usize;
        visit(role_count as u64)?;
        for (role, posture, name, _) in correspondence.native_witness_roles() {
            let text = role.len().checked_add(name.len()).ok_or_else(overflow)?;
            names_bytes = names_bytes.checked_add(text).ok_or_else(overflow)?;
            visit(text as u64 + 1)?;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                return Ok(None);
            }
            let Some(_) = layout.entity_kind(name) else {
                return Ok(None);
            };
            for _ in layout.native_output_aspects(name) {
                visit(1)?;
                aspect_count = aspect_count.checked_add(1).ok_or_else(overflow)?;
            }
        }
        let bytes = role_count
            .checked_mul(std::mem::size_of::<RoleProbe>())
            .and_then(|bytes| {
                aspect_count
                    .checked_mul(std::mem::size_of::<AspectProbe>())
                    .and_then(|aspects| bytes.checked_add(aspects))
            })
            .and_then(|bytes| bytes.checked_add(names_bytes))
            .and_then(|bytes| {
                bytes.checked_add(
                    std::mem::size_of::<OnceLock<Self>>() + 2 * std::mem::size_of::<usize>(),
                )
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        let memory = match execution {
            Some(execution) => {
                crate::domain_computation::primary_graph::structural_walk::memory::reserve(
                    execution,
                    execution.has_accounting(),
                    bytes,
                )
                .map_err(CompanionPreflightStop::Execution)?
            }
            None => None,
        };
        visit(role_count as u64 + aspect_count as u64)?;
        let mut roles = Vec::with_capacity(role_count);
        let mut aspects = Vec::with_capacity(aspect_count);
        for (role, posture, name, entity) in correspondence.native_witness_roles() {
            let kind = layout
                .entity_kind(name)
                .expect("validated checkpoint output kind");
            let first_aspect = aspects.len();
            // Each slot stays unfilled until its original revision is read.
            aspects.extend(
                layout
                    .native_output_aspects(name)
                    .map(|aspect| AspectProbe {
                        aspect: aspect.clone(),
                        revision: None,
                    }),
            );
            roles.push(RoleProbe {
                role: role.to_owned(),
                entity_name: name.to_owned(),
                kind,
                posture,
                retirement: None,
                entity: Some(entity),
                first_aspect,
                end_aspect: aspects.len(),
            });
        }
        let index = CheckpointRoleIndex::build(
            execution,
            visit,
            correspondence
                .native_witness_roles()
                .map(|(_, _, _, entity)| entity),
        )?;
        if !index.record_facts(facts, &roles, &mut aspects, visit)? {
            return Ok(None);
        }
        Ok(Some(Self {
            roles,
            aspects,
            _memory: memory,
        }))
    }
}

mod court_doors_for_test;
