//! Root occurrence issuance and current-loan scalar admission.
use super::*;
use crate::contracts::derived_relation::read_shape::DerivedRelationReadShape;
use crate::contracts::identity::data::primitives::KindId;

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
    pub(in crate::internal_port::validation::candidate_source) projection:
        ManagedIndexSourceProjection<'a>,
    pub(in crate::internal_port::validation::candidate_source) scope: EntityId,
}

impl ManagedIndexSourceRows<'_> {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        self.projection.program
    }
    pub const fn scope(&self) -> EntityId {
        self.scope
    }
    /// Cold acquisition is explicit; warm maintenance uses changed occurrences.
    pub fn acquire_all(
        &self,
    ) -> Result<super::super::projection::PreparedManagedIndexRelationRead, ReadDenial> {
        self.projection.prepare(
            &self.projection.program.owner_field_0().path,
            self.scope,
            DerivedRelationReadShape::Adjacency { reverse: false },
        )
    }
    /// Complete physical membership support, including negative adjacency reads.
    /// The prepared read must belong to this exact compiled Source loan.
    pub fn membership_support<'a>(
        &'a self,
        read: &'a super::super::projection::PreparedManagedIndexRelationRead,
    ) -> Result<&'a crate::contracts::derived_relation::read::DerivedRelationData, ReadDenial> {
        self.projection.read(
            read,
            &self.projection.program.owner_field_0().path,
            self.scope,
            DerivedRelationReadShape::Adjacency { reverse: false },
        )
    }
    pub fn occurrences(
        &self,
        read: &super::super::projection::PreparedManagedIndexRelationRead,
    ) -> Result<Vec<ManagedIndexSourceOccurrence>, ReadDenial> {
        let data = self.projection.read(
            read,
            &self.projection.program.owner_field_0().path,
            self.scope,
            DerivedRelationReadShape::Adjacency { reverse: false },
        )?;
        self.projection.claim_retained(
            data.endpoints()
                .len()
                .checked_mul(size_of::<ManagedIndexSourceOccurrence>())
                .ok_or(Definition::WorkBudgetExceeded)? as u64,
        )?;
        let mut occurrences = Vec::new();
        occurrences
            .try_reserve_exact(data.endpoints().len())
            .map_err(|_| {
                ReadDenial::Request(
                    crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
                )
            })?;
        for row in data.endpoints() {
            self.projection.checkpoint(1)?;
            occurrences.push(ManagedIndexSourceOccurrence {
                program: self.projection.program.clone(),
                scope: self.scope,
                row: row.entity().ok_or(Definition::InvalidMemberDomain)?,
            });
        }
        Ok(occurrences)
    }
    /// Reacquire actual inverse membership; a coordinate alone never admits a row.
    pub fn occurrence(
        &self,
        row: EntityId,
    ) -> Result<Option<ManagedIndexSourceOccurrence>, ReadDenial> {
        Ok(self
            .prepare_occurrence(row)?
            .map(|prepared| prepared.occurrence))
    }
    /// Retain the actual inverse read for current-loan projection consumers.
    pub fn prepare_occurrence(
        &self,
        row: EntityId,
    ) -> Result<Option<super::PreparedManagedIndexSourceOccurrence>, ReadDenial> {
        match self.projection.entity_kind(row) {
            Err(ReadDenial::Definition(Definition::RecordUnavailable)) => return Ok(None),
            Err(ReadDenial::Structural(
                crate::validation::custom_rule::StructuralWalkDenialKind::Read(
                    crate::contracts::validation::data::structural_read::StructuralReadError::RecordUnavailable,
                ),
            )) => return Ok(None),
            Err(cause) => return Err(cause),
            Ok(kind) if kind != self.projection.program.row_kind() => return Ok(None),
            Ok(_) => {}
        }
        let shape = DerivedRelationReadShape::Adjacency { reverse: true };
        let path = &self.projection.program.owner_field_0().path;
        let read = self.projection.prepare(path, row, shape)?;
        let data = self.projection.read(&read, path, row, shape)?;
        Ok(data
            .endpoints()
            .iter()
            .any(|root| root.entity() == Some(self.scope))
            .then(|| super::PreparedManagedIndexSourceOccurrence {
                occurrence: ManagedIndexSourceOccurrence {
                    program: self.projection.program.clone(),
                    scope: self.scope,
                    row,
                },
                membership: super::row_read::OccurrenceMembership::Inverse(read),
            }))
    }
    /// Field borrows cannot escape the actual loan's exact state callback.
    pub fn with_field<T>(
        &self,
        occurrence: &ManagedIndexSourceOccurrence,
        column: usize,
        project: impl FnOnce(crate::contracts::visibility::materialization::read_records::projection::projection_records::field_observation::RelationalProjectedFieldObservation<'_>) -> T,
    ) -> Result<T, ReadDenial> {
        self.prepare_row(occurrence)?.with_field(column, project)
    }

    pub(super) fn admit_occurrence(
        &self,
        occurrence: &ManagedIndexSourceOccurrence,
    ) -> Result<(), ReadDenial> {
        self.prepare_row(occurrence).map(|_| ())
    }
}

#[doc(hidden)]
pub trait ManagedIndexSourceRowsOwnerPort {
    fn entity_kind(&self, entity: EntityId) -> Result<KindId, ReadDenial>;
}

impl ManagedIndexSourceRowsOwnerPort for ManagedIndexSourceRows<'_> {
    fn entity_kind(&self, entity: EntityId) -> Result<KindId, ReadDenial> {
        self.projection.entity_kind(entity)
    }
}
