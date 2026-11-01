//! Genuine historical weak material; opaque acquisition is an outside-guard handoff.
use super::super::{
    ordered_visit, run, visit, RelationalAuthoredWriteDenial as Denial,
    RelationalAuthoredWriteOccurrence,
};
use crate::history::data::{CanonicalCommitEnvelope, LiveManagedSourceHistory};
use crate::identity::data::EntityId;
use crate::runtime::RelationalRuntime;
use crate::validation::candidate_source::{
    authored_history::{
        material::HistoricalSourcePair, ManagedSourceImpactKey, ManagedSourceImpactRows,
    },
    opaque_custody::{FundedSourceMaterialReceipt, FundedSourceMaterialShare},
    ManagedIndexSourceProgram,
};
use std::{any::Any, sync::Arc};
use worth_execution::ExecutionRequest;

pub enum RelationalAuthoredManagedSourceImpact {
    Unaffected,
    Conservative,
    Retained(RelationalAuthoredManagedSourceMaterial),
}
/// Owned sealed metadata only. This never keeps an opaque Source payload alive.
/// The actual canonical owner, program/scope and affected row cells remain bound.
pub struct RelationalAuthoredManagedSourceMaterial {
    _canonical: Arc<CanonicalCommitEnvelope>,
    program: ManagedIndexSourceProgram,
    scope: EntityId,
    rows: ManagedSourceImpactRows,
    before: FundedSourceMaterialReceipt,
    after: FundedSourceMaterialReceipt,
}
/// Actual strong opaque shares with their sealed funding. Acquire, use and
/// destroy this carrier OUTSIDE Native/runtime/attempt gates, even on refusal.
/// It grants no historical READ or Query correction permission.
pub struct RelationalAuthoredManagedSourcePayloads {
    before: FundedSourceMaterialShare,
    after: FundedSourceMaterialShare,
}
impl RelationalAuthoredManagedSourcePayloads {
    pub fn before(&self) -> &(dyn Any + Send + Sync) {
        self.before.value()
    }
    pub fn after(&self) -> &(dyn Any + Send + Sync) {
        self.after.value()
    }
}
impl RelationalAuthoredManagedSourceMaterial {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        &self.program
    }
    pub const fn scope(&self) -> EntityId {
        self.scope
    }
    /// Admit the actual roster count before traversing or looking up its rows.
    pub fn row_count(&self) -> usize {
        self.rows.0.len()
    }
    pub fn rows(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.rows.0.iter().map(|(row, _)| *row)
    }
    /// Call only AFTER leaving Native/runtime/attempt gates. Expired actual
    /// material is Conservative, never an empty Source. Both partial and full
    /// strong custody, including cancellation disposal, stay with this caller.
    pub fn acquire_payloads(
        &self,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<Option<RelationalAuthoredManagedSourcePayloads>, Denial> {
        // No opaque upgrade is performed within the Native work callback.
        run(execution, |context| {
            visit(context)?;
            visit(context)
        })?;
        let before = self.before.upgrade();
        let after = self.after.upgrade();
        execution.consult().map_err(Denial::Request)?;
        let (Some(before), Some(after)) = (before, after) else {
            return Ok(None);
        };
        Ok(Some(RelationalAuthoredManagedSourcePayloads {
            before,
            after,
        }))
    }
}

pub(crate) fn source_impact(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    observation: &crate::mvcc::ManagedIndexSourceObservation,
    occurrence: &RelationalAuthoredWriteOccurrence,
) -> Result<RelationalAuthoredManagedSourceImpact, Denial> {
    let selector = observation.authored_history_selector(runtime, execution)?;
    run(execution, |context| {
        visit(context)?;
        let packet = LiveManagedSourceHistory::verified(&occurrence.canonical)
            .ok_or(Denial::IncompleteManagedSourceHistory)?;
        if packet.runtime != runtime.runtime_instance_id() {
            return Err(Denial::ForeignOwner);
        }
        if !Arc::ptr_eq(&packet.catalog, &selector.catalog) {
            return Err(Denial::ForeignManagedSourceProgram);
        }
        let material = LiveManagedSourceHistory::verified_material(&occurrence.canonical)
            .ok_or(Denial::IncompleteManagedSourceHistory)?;
        let Some(owner) = &material.owner else {
            return Ok(RelationalAuthoredManagedSourceImpact::Conservative);
        };
        if !owner.admits(observation) {
            return Err(Denial::ForeignOwner);
        }
        let key = ManagedSourceImpactKey {
            program: selector.program,
            scope: selector.scope,
        };
        ordered_visit(context, packet.impacts.len(), size_of_val(&key) as u64)?;
        let Some(rows) = packet.impacts.get(&key) else {
            return Ok(RelationalAuthoredManagedSourceImpact::Unaffected);
        };
        ordered_visit(context, material.scopes.len(), size_of_val(&key) as u64)?;
        let pair = material
            .scopes
            .get(&key)
            .ok_or(Denial::IncompleteManagedSourceHistory)?;
        match pair {
            HistoricalSourcePair::Conservative => {
                Ok(RelationalAuthoredManagedSourceImpact::Conservative)
            }
            HistoricalSourcePair::Complete { before, after } => {
                visit(context)?;
                let program = packet
                    .catalog
                    .program(selector.program)
                    .ok_or(Denial::ForeignManagedSourceProgram)?;
                Ok(RelationalAuthoredManagedSourceImpact::Retained(
                    RelationalAuthoredManagedSourceMaterial {
                        _canonical: occurrence.canonical.clone(),
                        program: program.clone(),
                        scope: selector.scope,
                        rows: rows.clone(),
                        before: before.clone(),
                        after: after.clone(),
                    },
                ))
            }
        }
    })
}
