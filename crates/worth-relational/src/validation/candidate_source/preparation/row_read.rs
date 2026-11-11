//! Field reads carry the actual current-loan membership ticket.
use super::super::projection::PreparedManagedIndexRelationRead;
use super::*;
use crate::derived_relation::{DerivedRelationData, DerivedRelationReadShape as Shape};
use crate::graph_member::NativeMemberKey;

pub(super) struct OrderedStepSupport {
    pub(super) identity: crate::derived_relation::DerivedRelationReadIdentity,
    pub(super) members: std::collections::BTreeSet<EntityId>,
    pub(super) _records: (
        Vec<crate::validation::data::StructuralRelationRecord>,
        Vec<crate::validation::data::StructuralRelationRecord>,
    ),
    pub(super) _retention: crate::validation::CustomInvariantWorkMeter,
}
pub(super) enum OccurrenceMembership {
    Inverse(PreparedManagedIndexRelationRead),
    OrderedStep(Arc<OrderedStepSupport>),
}
enum Membership<'read> {
    Step {
        _prepared: &'read PreparedManagedIndexSourceOccurrence,
    },
    Borrowed {
        _read: &'read PreparedManagedIndexRelationRead,
    },
    Owned {
        _read: PreparedManagedIndexRelationRead,
    },
}
/// Actual inverse membership issued by one immutable Native loan. Its coordinate
/// is descriptive; consumption still validates the complete current read owner.
pub struct PreparedManagedIndexSourceOccurrence {
    pub(super) occurrence: ManagedIndexSourceOccurrence,
    pub(super) membership: OccurrenceMembership,
}
impl PreparedManagedIndexSourceOccurrence {
    pub fn occurrence(&self) -> &ManagedIndexSourceOccurrence {
        &self.occurrence
    }
    pub fn entity(&self) -> EntityId {
        self.occurrence.entity()
    }
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
            _membership: Membership::Borrowed { _read: read },
        })
    }
    /// Consume actual inverse membership under this exact current loan.
    pub fn prepare_row_from_occurrence<'read>(
        &'read self,
        prepared: &'read PreparedManagedIndexSourceOccurrence,
        occurrence: &'read ManagedIndexSourceOccurrence,
    ) -> Result<PreparedManagedIndexSourceRowRead<'read, 'source>, ReadDenial> {
        self.check_row_context(occurrence)?;
        self.check_row_context(&prepared.occurrence)?;
        if occurrence.row != prepared.occurrence.row {
            return Err(Definition::ForeignObservation.into());
        }
        let membership = match &prepared.membership {
            OccurrenceMembership::Inverse(read) => {
                let data = self.projection.read(
                    read,
                    &self.projection.program.0.path,
                    occurrence.row,
                    Shape::Adjacency { reverse: true },
                )?;
                self.require_member(data, self.scope)?;
                Membership::Borrowed { _read: read }
            }
            OccurrenceMembership::OrderedStep(support) => {
                let levels = usize::BITS - support.members.len().max(1).leading_zeros();
                self.projection.checkpoint(u64::from(levels) + 1)?;
                if self.projection.scope_identity(self.scope)?.as_ref() != Some(&support.identity)
                    || !support.members.contains(&occurrence.row)
                {
                    return Err(Definition::ForeignObservation.into());
                }
                Membership::Step {
                    _prepared: prepared,
                }
            }
        };
        Ok(PreparedManagedIndexSourceRowRead {
            rows: self,
            occurrence,
            _membership: membership,
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
            _membership: Membership::Owned { _read: read },
        })
    }
}
impl PreparedManagedIndexSourceRowRead<'_, '_> {
    /// Carry this actual row admission into its declared endpoint projection.
    pub fn prepare_projection(
        &self,
        route: usize,
    ) -> Result<super::PreparedManagedIndexProjectionRead, ReadDenial> {
        self.rows
            .prepare_admitted_projection(self.occurrence, route)
    }

    /// Retain only content selected through this exact field and membership ticket.
    /// The observation preserves absent aspect, absent field and present value;
    /// Query owns required/optional interpretation and scalar decoding.
    pub fn with_field_content<T>(
        &self,
        column: usize,
        project: impl FnOnce(
            crate::runtime::RelationalProjectedFieldObservation<'_>,
            Option<crate::content::RelationalContentFrame>,
        ) -> T,
    ) -> Result<T, super::super::ManagedIndexSourcePreparationDenial> {
        self.rows.projection.checkpoint(0)?;
        let field = self
            .rows
            .program()
            .fields()
            .get(column)
            .ok_or(Definition::InvalidExpression)?;
        self.rows.projection.with_field_content_at(
            self.occurrence.row,
            self.rows.program().row_kind(),
            field,
            project,
        )
    }
    /// The field borrows its own actual loan; no caller can substitute another view.
    pub fn with_field<T>(
        &self,
        column: usize,
        project: impl FnOnce(crate::runtime::RelationalProjectedFieldObservation<'_>) -> T,
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
