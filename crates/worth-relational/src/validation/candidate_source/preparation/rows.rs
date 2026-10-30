//! Root occurrence issuance and current-loan scalar admission.
use super::*;
use crate::derived_relation::DerivedRelationReadShape;

/// Full-generation occurrence issued by an actual membership acquisition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedIndexSourceOccurrence {
    pub(super) program: ManagedIndexSourceProgram,
    pub(super) scope: EntityId,
    pub(super) row: EntityId,
}

impl ManagedIndexSourceOccurrence {
    pub const fn entity(&self) -> EntityId {
        self.row
    }
    pub const fn scope(&self) -> EntityId {
        self.scope
    }
}

pub struct ManagedIndexSourceRows<'a> {
    pub(in crate::validation::candidate_source) projection: ManagedIndexSourceProjection<'a>,
    pub(in crate::validation::candidate_source) scope: EntityId,
}

impl ManagedIndexSourceRows<'_> {
    /// Cold acquisition is explicit; warm maintenance uses changed occurrences.
    pub fn acquire_all(
        &self,
    ) -> Result<crate::derived_relation::PreparedDerivedRelationRead, ReadDenial> {
        self.projection.relations.prepare_derived_source(
            &self.projection.program.0.path,
            self.scope,
            DerivedRelationReadShape::Adjacency { reverse: false },
        )
    }

    pub fn occurrences(
        &self,
        read: &crate::derived_relation::PreparedDerivedRelationRead,
    ) -> Result<Vec<ManagedIndexSourceOccurrence>, ReadDenial> {
        if read.identity()
            != &self.projection.relations.derived_source_identity(
                &self.projection.program.0.path,
                self.scope,
                DerivedRelationReadShape::Adjacency { reverse: false },
            )?
        {
            return Err(Definition::ForeignObservation.into());
        }
        let data = self.projection.relations.read_prepared_derived_source(
            read,
            DerivedRelationReadShape::Adjacency { reverse: false },
        )?;
        self.projection
            .relations
            .read_budget()
            .claim_retained(
                data.endpoints()
                    .len()
                    .checked_mul(size_of::<ManagedIndexSourceOccurrence>())
                    .ok_or(Definition::WorkBudgetExceeded)? as u64,
            )
            .map_err(ReadDenial::Structural)?;
        data.endpoints()
            .iter()
            .map(|row| {
                self.projection
                    .relations
                    .read_budget()
                    .checkpoint(1)
                    .map_err(ReadDenial::Structural)?;
                Ok(ManagedIndexSourceOccurrence {
                    program: self.projection.program.clone(),
                    scope: self.scope,
                    row: row.entity().ok_or(Definition::InvalidMemberDomain)?,
                })
            })
            .collect()
    }

    /// Reacquire one full-generation occurrence through this actual audience's
    /// inverse Source path. An entity identifier alone does not admit a row.
    pub fn occurrence(
        &self,
        row: EntityId,
    ) -> Result<Option<ManagedIndexSourceOccurrence>, ReadDenial> {
        use crate::validation::data::StructuralReadError;
        match self.projection.relations.entity_kind(row) {
            Err(StructuralReadError::RecordUnavailable) => return Ok(None),
            Err(cause) => {
                return Err(ReadDenial::Structural(
                    self.projection.relations.read_budget().refusal(cause),
                ))
            }
            Ok(kind) if kind != self.projection.program.row_kind() => return Ok(None),
            Ok(_) => {}
        }
        let read = self.projection.relations.prepare_derived_source(
            &self.projection.program.0.path,
            row,
            DerivedRelationReadShape::Adjacency { reverse: true },
        )?;
        let data = self.projection.relations.read_prepared_derived_source(
            &read,
            DerivedRelationReadShape::Adjacency { reverse: true },
        )?;
        Ok(data
            .endpoints()
            .iter()
            .any(|root| root.entity() == Some(self.scope))
            .then(|| ManagedIndexSourceOccurrence {
                program: self.projection.program.clone(),
                scope: self.scope,
                row,
            }))
    }

    /// An old occurrence is never sufficient membership or observation proof.
    /// Reacquire the inverse path in this actual loan before projecting values.
    pub fn field(
        &self,
        occurrence: &ManagedIndexSourceOccurrence,
        column: usize,
    ) -> Result<Option<&worth_foundational::facade::AspectValue>, ReadDenial> {
        self.admit_occurrence(occurrence)?;
        self.projection.field(occurrence.row, column)
    }

    pub(super) fn admit_occurrence(
        &self,
        occurrence: &ManagedIndexSourceOccurrence,
    ) -> Result<(), ReadDenial> {
        if occurrence.program != *self.projection.program || occurrence.scope != self.scope {
            return Err(Definition::ForeignDefinition.into());
        }
        let read = self.projection.relations.prepare_derived_source(
            &self.projection.program.0.path,
            occurrence.row,
            DerivedRelationReadShape::Adjacency { reverse: true },
        )?;
        let data = self.projection.relations.read_prepared_derived_source(
            &read,
            DerivedRelationReadShape::Adjacency { reverse: true },
        )?;
        if !data
            .endpoints()
            .iter()
            .any(|root| root.entity() == Some(self.scope))
        {
            return Err(Definition::RecordUnavailable.into());
        }
        Ok(())
    }
}
