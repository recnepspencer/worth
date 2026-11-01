//! Field reads carry the actual current-loan membership ticket.
use super::super::projection::PreparedManagedIndexRelationRead;
use super::*;
use crate::derived_relation::{DerivedRelationData, DerivedRelationReadShape as Shape};
use crate::graph_member::NativeMemberKey;

enum Membership<'read> {
    Forward {
        _read: &'read PreparedManagedIndexRelationRead,
    },
    Inverse {
        _read: PreparedManagedIndexRelationRead,
    },
}
/// Borrow-confined physical row preparation, not principal READ permission.
/// The exact loan and successful membership acquisition remain owned here.
pub struct PreparedManagedIndexSourceRowRead<'read, 'source> {
    rows: &'read ManagedIndexSourceRows<'source>,
    occurrence: &'read ManagedIndexSourceOccurrence,
    _membership: Membership<'read>,
}
impl<'source> ManagedIndexSourceRows<'source> {
    fn check_row_context(
        &self,
        occurrence: &ManagedIndexSourceOccurrence,
    ) -> Result<(), ReadDenial> {
        self.projection.checkpoint(1)?;
        if occurrence.program != *self.projection.program || occurrence.scope != self.scope {
            return Err(Definition::ForeignDefinition.into());
        }
        Ok(())
    }
    fn require_member(
        &self,
        data: &DerivedRelationData,
        member: EntityId,
    ) -> Result<(), ReadDenial> {
        let levels = u64::from(usize::BITS - data.endpoints().len().max(1).leading_zeros()) + 1;
        self.projection
            .checkpoint(levels * size_of::<NativeMemberKey>() as u64)?;
        if data
            .endpoints()
            .binary_search(&NativeMemberKey::Entity(member))
            .is_err()
        {
            return Err(Definition::RecordUnavailable.into());
        }
        self.projection.checkpoint(0)
    }
    /// Validate the actual current forward acquisition, including this complete row.
    pub fn prepare_row_from_read<'read>(
        &'read self,
        read: &'read PreparedManagedIndexRelationRead,
        occurrence: &'read ManagedIndexSourceOccurrence,
    ) -> Result<PreparedManagedIndexSourceRowRead<'read, 'source>, ReadDenial> {
        self.check_row_context(occurrence)?;
        let data = self.projection.read(
            read,
            &self.projection.program.0.path,
            self.scope,
            Shape::Adjacency { reverse: false },
        )?;
        self.require_member(data, occurrence.row)?;
        Ok(PreparedManagedIndexSourceRowRead {
            rows: self,
            occurrence,
            _membership: Membership::Forward { _read: read },
        })
    }
    /// A retained occurrence has no current membership ticket. Reacquire it once.
    pub fn prepare_row<'read>(
        &'read self,
        occurrence: &'read ManagedIndexSourceOccurrence,
    ) -> Result<PreparedManagedIndexSourceRowRead<'read, 'source>, ReadDenial> {
        self.check_row_context(occurrence)?;
        let path = &self.projection.program.0.path;
        let shape = Shape::Adjacency { reverse: true };
        let read = self.projection.prepare(path, occurrence.row, shape)?;
        let data = self.projection.read(&read, path, occurrence.row, shape)?;
        self.require_member(data, self.scope)?;
        Ok(PreparedManagedIndexSourceRowRead {
            rows: self,
            occurrence,
            _membership: Membership::Inverse { _read: read },
        })
    }
}
impl PreparedManagedIndexSourceRowRead<'_, '_> {
    /// The field borrows its own actual loan; no caller can substitute another view.
    pub fn with_field<T>(
        &self,
        column: usize,
        project: impl FnOnce(Option<&worth_foundational::facade::AspectValue>) -> T,
    ) -> Result<T, ReadDenial> {
        self.rows.projection.checkpoint(0)?;
        // Construction proved membership in this immutable loan. The retained
        // actual ticket keeps its support funded; each field still consumes
        // this loan's audience, kind and original controls below.
        let field = self
            .rows
            .program()
            .fields()
            .get(column)
            .ok_or(Definition::InvalidExpression)?;
        self.rows.projection.with_field_at(
            self.occurrence.row,
            self.rows.program().row_kind(),
            field,
            project,
        )
    }
}
