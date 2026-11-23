//! A successor round's applied completion read through its own-output view
//! (PR C §1.5, #277): field and aspect-presence reads of a covered record are
//! read at the chain head's pre-pin; relation and adjacency reads are served
//! (`OwnOutputOverlay::serve_fact`). A covered entity absent at the head,
//! read directly or reached through a link the chain did not make, marks the
//! view incoherent.
use super::super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationFactKey as Key, WorthQueryApplicationObservedFact as Fact,
};
use super::WorthQueryApplicationSnapshotLease;
use worth_relational::facade::identity::EntityId;

impl WorthQueryApplicationSnapshotLease {
    /// Observe `key` at the applied image, when the lease has one, through a
    /// successor's own-output view.
    pub fn observe_applied(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: &Key,
    ) -> Option<Result<Fact, Denial>> {
        use super::super::read_set::fact_observation::observe_applied_fact;
        let entity = single_record(key);
        let pin = self.applied_projection_of(entity)?;
        let mut observed = observe_applied_fact(execution, &self.layout, key, pin);
        let Some(own) = &self.own_outputs else {
            return Some(observed);
        };
        if let Ok(fact) = observed {
            observed = own.serve_fact(execution, fact, || {
                observe_applied_fact(execution, &self.layout, key, own.head())
            });
        }
        // Incoherent only when the entity itself is absent at the head pre-pin,
        // never for an absent aspect or field of an entity that exists there.
        if let (Err(denial), Some(entity)) = (&observed, entity)
            && denial.kind() == Kind::MissingAuthoritativeFact
            && own.covers(entity)
            && self.absent_at_head(execution, key, entity, pin)
        {
            own.mark_incoherent(entity);
        }
        Some(observed)
    }

    /// Whether `entity` is absent at the head pre-pin, judged by the entity
    /// read the field or aspect key names.
    fn absent_at_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: &Key,
        entity: EntityId,
        pin: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> bool {
        let (Key::Field { entity: name, .. } | Key::AspectPresence { entity: name, .. }) = key
        else {
            return false;
        };
        let presence = Key::Entity {
            entity: name.clone(),
            entity_id: entity,
        };
        super::super::read_set::fact_observation::observe_applied_fact(
            execution,
            &self.layout,
            &presence,
            pin,
        )
        .is_err_and(|denial| {
            denial.kind() == Kind::MissingAuthoritativeFact
                && denial.relational_execution_cause().is_none()
        })
    }
}

/// The one record a field or aspect-presence key reads; the own-output view
/// serves it at the chain head's pre-pin when the chain wrote it.
fn single_record(key: &Key) -> Option<EntityId> {
    match key {
        Key::Field { entity_id, .. } | Key::AspectPresence { entity_id, .. } => Some(*entity_id),
        _ => None,
    }
}
