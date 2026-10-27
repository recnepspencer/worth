//! Recover the original output expectations, never expectations from a new head.
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;

use super::*;
use crate::domain_computation::primary_graph::application_attempt::Movement;

mod fact_index;
#[cfg(test)]
mod tests;
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
        let role_count = correspondence.native_witness_roles().len();
        if role_count == 0 {
            return Ok(None);
        }
        let mut aspect_count = 0usize;
        for (_, posture, name, _) in correspondence.native_witness_roles() {
            if posture == WorthQueryApplicationOutputPosture::Retire {
                return Ok(None);
            }
            let Some(_) = layout.entity_kind(name) else {
                return Ok(None);
            };
            for _ in layout.native_output_aspects(name) {
                aspect_count = aspect_count.checked_add(1).ok_or_else(overflow)?;
            }
        }
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
            correspondence
                .native_witness_roles()
                .map(|(_, _, _, entity)| entity),
        )?;
        if !index.record_facts(facts, &roles, &mut aspects)? {
            return Ok(None);
        }
        let cell = Arc::new(OnceLock::new());
        cell.set(Self { roles, aspects })
            .expect("new checkpoint witness cell is empty");
        Ok(Some(cell))
    }
}
