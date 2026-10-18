//! Recover the original output expectations, never expectations from a new head.
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;

use super::*;
use crate::domain_computation::primary_graph::application_attempt::Movement;

mod fact_index;
#[cfg(test)]
mod tests;
mod work_bound;
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, super::super::invalidation::SettlementVerificationStop> {
        if !self.unchanged_in(relational, snapshot, admission)? {
            return Ok(false);
        }
        for fact in facts.iter() {
            admission.charge_external_work(1)?;
            match fact {
                Fact::SourceEntity { .. }
                | Fact::SourceFieldRevision { .. }
                | Fact::Entity { .. }
                | Fact::SourceAdjacencyRevision { .. }
                | Fact::SourceAspectRevision { .. }
                | Fact::IndexedEntitySelection { .. } => {}
                _ => return Ok(false),
            }
            // The fact owner reserves on this carried meter before reading
            // and settles the actual probe debit on success or failure.
            match fact.source_currentness_in(relational, execution, snapshot, admission)? {
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
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Arc<OnceLock<Self>>>, CompanionPreflightStop> {
        admission.charge_external_work(5)?;
        let role_count = correspondence.native_witness_roles().len();
        if role_count == 0 {
            return Ok(None);
        }
        let mut aspect_count = 0usize;
        let mut name_bytes = 0u64;
        for (role, posture, name, _) in correspondence.native_witness_roles() {
            admission.charge_external_work(1)?;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                return Ok(None);
            }
            prepay_catalog(layout, name, admission)?;
            let Some(_) = layout.entity_kind(name) else {
                return Ok(None);
            };
            name_bytes = name_bytes
                .checked_add(role.len() as u64)
                .and_then(|n| n.checked_add(name.len() as u64))
                .ok_or_else(overflow)?;
            for aspect in layout.native_output_aspects(name) {
                admission.charge_external_work(1)?;
                aspect_count = aspect_count.checked_add(1).ok_or_else(overflow)?;
                name_bytes = name_bytes
                    .checked_add(aspect.as_str().len() as u64)
                    .ok_or_else(overflow)?;
            }
        }
        let backing = role_count
            .checked_mul(size_of::<RoleProbe>())
            .and_then(|n| n.checked_add(aspect_count.checked_mul(size_of::<AspectProbe>())?))
            .and_then(|n| u64::try_from(n).ok())
            .and_then(|n| n.checked_add(name_bytes))
            .ok_or_else(overflow)?;
        let visits = role_count.checked_add(aspect_count).ok_or_else(overflow)? as u64;
        let retained = owner.retain_native_output_witness::<Self>(
            backing,
            name_bytes.checked_add(visits).ok_or_else(overflow)?,
            admission,
        )?;
        let mut roles = Vec::with_capacity(role_count);
        let mut aspects = Vec::with_capacity(aspect_count);
        for (role, posture, name, entity) in correspondence.native_witness_roles() {
            prepay_catalog(layout, name, admission)?;
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
            admission,
        )?;
        if !index.record_facts(facts, &roles, &mut aspects, admission)? {
            return Ok(None);
        }
        let cell = Arc::new(OnceLock::new());
        cell.set(Self {
            roles,
            aspects,
            _retained: retained,
        })
        .expect("new checkpoint witness cell is empty");
        Ok(Some(cell))
    }
}

fn prepay_catalog(
    layout: &WorthQueryPrimaryGraphLayout,
    name: &str,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    admission.charge_external_work(3)?;
    let lookup = layout
        .native_output_lookup_work(name)
        .ok_or_else(overflow)?;
    admission.charge_external_work(lookup)
}
