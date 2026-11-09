//! The installed event recipe alone selects the predecessor and its step changes.
use super::*;
use crate::derived_relation::{
    evaluation::SourceRead, DerivedRelationAdjacency, DerivedRelationReads, StructuralSource,
};
use crate::identity::data::KindId;
use crate::validation::data::StructuralRelationRecord;

impl ManagedIndexSourceScope<'_> {
    pub fn ordered_predecessor(&self) -> Result<Option<EntityId>, Denial> {
        let Some(recipe) = self.program.0.ordered else {
            return Ok(None);
        };
        if !self.scope_exists()? {
            return Ok(None);
        }
        let records = self.event_records(recipe.predecessor)?;
        match records.as_slice() {
            [] => Ok(None),
            [record] if record.target != self.scope => Ok(Some(record.target)),
            _ => Err(Definition::InvalidEventChain.into()),
        }
    }

    /// An exact retained predecessor proves its prefix. Only this event's
    /// physical leaves are acquired; a descriptive parent coordinate is insufficient.
    pub fn prepare_ordered_step(
        &self,
        predecessor: &Arc<dyn Any + Send + Sync>,
    ) -> Result<Vec<(EntityId, Option<PreparedManagedIndexSourceOccurrence>)>, Denial> {
        use super::row_read::{OccurrenceMembership, OrderedStepSupport};
        let recipe = self.program.0.ordered.ok_or(Denial::ForeignSource)?;
        let parent = self.ordered_predecessor()?.ok_or(Denial::ForeignSource)?;
        if !self.committed_available {
            return Err(Denial::RootUnavailable);
        }
        if self.closure.scopes.contains_key(&parent) {
            return Err(Denial::PredecessorUnavailable {
                predecessor: parent,
            });
        }
        let basis = self.interpretation.physical_basis();
        let image = basis
            .inner
            .root
            .managed_source_image
            .get()
            .ok_or(Denial::RootUnavailable)?;
        image.admit_predecessor_material(
            basis,
            self.program,
            parent,
            predecessor,
            self.interpretation.current_registration_generation()?,
        )?;
        let introduced = self.event_records(recipe.introduced)?;
        let retired = self.event_records(recipe.retired)?;
        let budget = self.read_budget();
        let mut members = std::collections::BTreeMap::new();
        for (records, live) in [(&introduced, true), (&retired, false)] {
            for record in records {
                budget
                    .checkpoint(u64::from(usize::BITS - members.len().max(1).leading_zeros()) + 1)
                    .map_err(ReadDenial::Structural)?;
                budget
                    .claim_retained((size_of::<(EntityId, bool)>() * 5) as u64)
                    .map_err(ReadDenial::Structural)?;
                members.insert(record.target, live);
            }
        }
        budget
            .claim_retained(
                (size_of::<OrderedStepSupport>()
                    + 2 * size_of::<usize>()
                    + members.len() * size_of::<EntityId>() * 5) as u64,
            )
            .map_err(ReadDenial::Structural)?;
        let after = self.proposed();
        let support = Arc::new(OrderedStepSupport {
            identity: after
                .projection
                .scope_identity(self.scope)?
                .ok_or(Denial::ForeignSource)?,
            members: members
                .iter()
                .filter_map(|(&row, &live)| live.then_some(row))
                .collect(),
            _records: (introduced, retired),
            _retention: self.views.after.work_meter(),
        });
        budget
            .claim_retained(
                (members.len()
                    * size_of::<(EntityId, Option<PreparedManagedIndexSourceOccurrence>)>())
                    as u64,
            )
            .map_err(ReadDenial::Structural)?;
        members
            .into_iter()
            .map(|(row, live)| {
                budget.checkpoint(1).map_err(ReadDenial::Structural)?;
                let present = if live {
                    match after.projection.entity_kind(row) {
                        Ok(kind) => kind == self.program.row_kind(),
                        Err(ReadDenial::Definition(Definition::RecordUnavailable))
                        | Err(ReadDenial::Structural(
                            crate::validation::data::StructuralWalkDenialKind::Read(
                                crate::validation::data::StructuralReadError::RecordUnavailable,
                            ),
                        )) => false,
                        Err(cause) => return Err(cause.into()),
                    }
                } else {
                    false
                };
                let occurrence = present.then(|| PreparedManagedIndexSourceOccurrence {
                    occurrence: ManagedIndexSourceOccurrence {
                        program: self.program.clone(),
                        scope: self.scope,
                        row,
                    },
                    membership: OccurrenceMembership::OrderedStep(Arc::clone(&support)),
                });
                Ok((row, occurrence))
            })
            .collect()
    }

    fn event_records(&self, relation: KindId) -> Result<Vec<StructuralRelationRecord>, Denial> {
        self.views
            .after
            .require_derived_relation(&self.program.0.path)?;
        let mut source = StructuralSource {
            view: &self.views.after,
            reads: DerivedRelationReads::default(),
        };
        Ok(source.adjacency(DerivedRelationAdjacency {
            anchor: self.scope,
            relation,
            outgoing: true,
        })?)
    }
}
