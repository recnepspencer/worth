//! A successor round's applied completion read through its own-output view
//! (PR C §1.5, #277 typed limit): a covered entity absent at the head, read
//! as a field or reached as a relation endpoint, marks the view incoherent.
use super::WorthQueryApplicationSnapshotLease;

impl WorthQueryApplicationSnapshotLease {
    /// Observe `key` at the applied image, when the lease has one. Only a
    /// field read is served by a successor's own-output view; a covered
    /// record absent there was created by the chain, and the view is
    /// incoherent (#277).
    pub fn observe_applied(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: &super::super::WorthQueryApplicationFactKey,
    ) -> Option<Result<super::super::WorthQueryApplicationObservedFact, super::super::WorthQueryApplicationAttemptDenial>>
    {
        let entity = match key {
            super::super::WorthQueryApplicationFactKey::Field { entity_id, .. } => Some(*entity_id),
            _ => None,
        };
        let pin = self.applied_projection_of(entity)?;
        let observed =
            super::super::read_set::fact_observation::observe_applied_fact(execution, &self.layout, key, pin);
        // A relation read at the current image whose endpoint the chain
        // created (absent at the head pre-pin) reads a key the chain wrote
        // outside the entity-only view (#277): mark it and fail, so the
        // round stops typed. Covered endpoints that exist at the head pass.
        if let (Ok(fact), Some(own)) = (&observed, &self.own_outputs) {
            let endpoints: Vec<worth_relational::facade::identity::EntityId> = match fact {
                super::super::WorthQueryApplicationObservedFact::Relation {
                    from,
                    to,
                    matching_relations,
                    ..
                } if !matching_relations.is_empty() => vec![*from, *to],
                super::super::WorthQueryApplicationObservedFact::Adjacency { relations, .. } => {
                    relations.iter().flat_map(|relation| [relation.from, relation.to]).collect()
                }
                _ => Vec::new(),
            };
            for endpoint in endpoints {
                match own.created_by_chain(execution, endpoint) {
                    Ok(false) => {}
                    Ok(true) => {
                        own.mark_incoherent(endpoint);
                        return Some(Err(super::super::WorthQueryApplicationAttemptDenial::new(
                            super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                            "own-output relation endpoint",
                        )));
                    }
                    Err(cause) => {
                        return Some(Err(super::super::WorthQueryApplicationAttemptDenial::relational_read_stopped(
                            cause,
                            "own-output relation endpoint",
                        )));
                    }
                }
            }
        }
        // Incoherent only when the entity itself is absent at the head pre-pin,
        // never for an absent aspect or field of an entity that exists there.
        if let (Err(denial), Some(entity), Some(own)) = (&observed, entity, &self.own_outputs)
            && denial.kind() == super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
            && own.covers(entity)
            && let super::super::WorthQueryApplicationFactKey::Field { entity: name, .. } = key
        {
            let presence = super::super::WorthQueryApplicationFactKey::Entity {
                entity: name.clone(),
                entity_id: entity,
            };
            if super::super::read_set::fact_observation::observe_applied_fact(execution, &self.layout, &presence, pin)
                .is_err_and(|denial| {
                    denial.kind() == super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
                        && denial.relational_execution_cause().is_none()
                })
            {
                own.mark_incoherent(entity);
            }
        }
        Some(observed)
    }
}
