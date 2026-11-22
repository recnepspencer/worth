//! Pending domain refusals (PR C §1.7): a route derivation that refuses in
//! its domain while a later authored stage remains keeps its proofs, the
//! decision's keyed reads, and waits. R2 retries it only when an append marks
//! one of them; a read without a key makes every append a possible change.
use super::*;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key::FactTouchKey;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey as Key;
use crate::domain_computation::primary_graph::invariant_projection::operation_reader::decision_reads::RefusalProofs;

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    /// A fresh read identity for a refused derivation's proofs.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn next_refusal_read(
        &self,
    ) -> ParentRead {
        self.next_read()
    }
    /// Retain a refused derivation's proofs as `read`: R2 retries its task
    /// when an append marks one.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn register_refusal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: ParentRead,
        proofs: &RefusalProofs,
    ) -> Result<(), CoupledSourceReadStop> {
        let subject = "pending refusal qualification";
        let graph = self.application.runtime.primary_provider.graph_ref();
        let layout = graph.layout_ref();
        let keys = if proofs.conservative {
            vec![None]
        } else {
            proofs
                .keys
                .iter()
                .map(|key| {
                    touch_keys(
                        key,
                        |entity| layout.entity_kind(entity),
                        |relation| layout.relation(relation).map(|relation| relation.kind),
                    )
                })
                .collect()
        };
        charge(execution, subject, keys.len() as u64 + 1)?;
        let inserted = self
            .ledger
            .borrow_mut()
            .retain(execution, Derivation::refusal(read, self.applied_position()), keys)
            .map_err(|stop| ledger_stop(subject, stop))?;
        charge(execution, subject, inserted)
    }
}

/// The touch keys an append changing `key` certifies, or `None` when no
/// narrower key exists (conservative).
fn touch_keys(
    key: &Key,
    entity_kind: impl Fn(&str) -> Option<worth_relational::facade::identity::KindId>,
    relation_kind: impl Fn(&str) -> Option<worth_relational::facade::identity::KindId>,
) -> Option<Vec<FactTouchKey>> {
    Some(match key {
        Key::Field {
            entity, locator, ..
        }
        | Key::AspectPresence {
            entity, locator, ..
        } => vec![FactTouchKey::Aspect(
            entity_kind(entity)?,
            locator.aspect().aspect_key().clone(),
        )],
        Key::Entity { entity, .. } => vec![FactTouchKey::Kind(entity_kind(entity)?)],
        Key::Relation { relation, .. } => vec![FactTouchKey::Relation(relation_kind(relation)?)],
        _ => return None,
    })
}
