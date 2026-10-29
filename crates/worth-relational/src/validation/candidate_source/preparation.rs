//! Borrow the applied Native Source; seal each root under that same custody.
use super::scope_closure::PreparedManagedIndexScopeClosure;
use super::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
    ManagedIndexSourceProjection,
};
use crate::derived_relation::{
    DerivedRelationDenial as Definition, DerivedRelationReadDenial as ReadDenial,
    DerivedRelationReadShape,
};
use crate::identity::data::EntityId;
use crate::validation::data::{StructuralAspectStateView, StructuralRelationView};
use std::{any::Any, sync::Arc};

pub(crate) type RootPayload = Arc<dyn Any + Send + Sync>;

/// Native's physical posture, never selected by a provider or Query boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedIndexSourceAvailability {
    Available,
    Unavailable,
}

pub(crate) struct SourceViews<'a> {
    pub(crate) before: StructuralRelationView<'a>,
    pub(crate) after: StructuralRelationView<'a>,
    pub(crate) before_fields: StructuralAspectStateView<'a>,
    pub(crate) after_fields: StructuralAspectStateView<'a>,
}

/// Native-created invocation scope. No constructor or caller root argument.
pub struct ManagedIndexSourceScope<'a> {
    pub(crate) program: &'a ManagedIndexSourceProgram,
    pub(crate) scope: EntityId,
    pub(crate) changed: &'a std::collections::BTreeSet<EntityId>,
    pub(crate) views: &'a SourceViews<'a>,
    pub(crate) availability: ManagedIndexSourceAvailability,
    pub(crate) committed_available: bool,
}

/// Full-generation occurrence issued by an actual membership acquisition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedIndexSourceOccurrence {
    program: ManagedIndexSourceProgram,
    scope: EntityId,
    row: EntityId,
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
    pub(super) projection: ManagedIndexSourceProjection<'a>,
    pub(super) scope: EntityId,
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

    pub(crate) fn occurrence(
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
        self.projection.field(occurrence.row, column)
    }
}

impl<'a> ManagedIndexSourceScope<'a> {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        self.program
    }
    pub const fn scope(&self) -> EntityId {
        self.scope
    }
    pub const fn physical_availability(&self) -> ManagedIndexSourceAvailability {
        self.availability
    }
    /// Retirement is an actual logical-view disposition, never missing-as-empty.
    pub fn scope_exists(&self) -> Result<bool, Denial> {
        self.scope_exists_in(&self.views.after)
    }
    pub fn committed_scope_exists(&self) -> Result<bool, Denial> {
        if !self.committed_available {
            return Err(Denial::RootUnavailable);
        }
        self.scope_exists_in(&self.views.before)
    }
    fn scope_exists_in(&self, view: &StructuralRelationView<'_>) -> Result<bool, Denial> {
        view.read_budget()
            .checkpoint(1)
            .map_err(ReadDenial::Structural)?;
        match view.entity_kind(self.scope) {
            Ok(kind) if kind == self.program.scope_kind() => Ok(true),
            Err(crate::validation::data::StructuralReadError::RecordUnavailable) => Ok(false),
            Ok(_) => Err(Denial::ForeignSource),
            Err(cause) => Err(ReadDenial::Structural(view.read_budget().refusal(cause)).into()),
        }
    }
    pub fn changed_entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.changed.iter().copied()
    }

    pub fn changed_occurrences(
        &self,
    ) -> Result<
        Vec<(
            Option<ManagedIndexSourceOccurrence>,
            Option<ManagedIndexSourceOccurrence>,
        )>,
        Denial,
    > {
        self.views
            .after
            .read_budget()
            .claim_retained(
                self.changed
                    .len()
                    .checked_mul(size_of::<(
                        Option<ManagedIndexSourceOccurrence>,
                        Option<ManagedIndexSourceOccurrence>,
                    )>())
                    .ok_or(ReadDenial::Definition(Definition::WorkBudgetExceeded))?
                    as u64,
            )
            .map_err(ReadDenial::Structural)?;
        let before = self.committed()?;
        let after = self.proposed();
        self.changed
            .iter()
            .map(|row| Ok((before.occurrence(*row)?, after.occurrence(*row)?)))
            .collect()
    }

    pub fn committed(&self) -> Result<ManagedIndexSourceRows<'_>, Denial> {
        if !self.committed_available {
            return Err(Denial::RootUnavailable);
        }
        Ok(self.rows(true))
    }
    pub fn proposed(&self) -> ManagedIndexSourceRows<'_> {
        self.rows(false)
    }

    fn rows(&self, before: bool) -> ManagedIndexSourceRows<'_> {
        ManagedIndexSourceRows {
            scope: self.scope,
            projection: ManagedIndexSourceProjection {
                program: self.program,
                relations: if before {
                    &self.views.before
                } else {
                    &self.views.after
                },
                aspects: if before {
                    &self.views.before_fields
                } else {
                    &self.views.after_fields
                },
                audience: self.program.0.issuer.admit(),
            },
        }
    }

    /// Sealing is candidate-local and does not install a root. The Native
    /// owner verifies the complete set before any dependent invariant runs.
    pub fn seal(
        self,
        committed: Option<RootPayload>,
        proposed: Option<RootPayload>,
    ) -> PreparedManagedIndexSourceRoot {
        PreparedManagedIndexSourceRoot {
            program: self.program.clone(),
            scope: self.scope,
            committed,
            proposed,
            availability: self.availability,
        }
    }
}

pub struct PreparedManagedIndexSourceRoot {
    pub(crate) program: ManagedIndexSourceProgram,
    pub(crate) scope: EntityId,
    pub(crate) committed: Option<RootPayload>,
    pub(crate) proposed: Option<RootPayload>,
    pub(crate) availability: ManagedIndexSourceAvailability,
}

pub(crate) struct PreparedSourceProgram {
    pub(crate) program: ManagedIndexSourceProgram,
    pub(crate) affected_scopes: std::collections::BTreeSet<EntityId>,
    pub(crate) closure: PreparedManagedIndexScopeClosure,
    pub(crate) roots: Vec<PreparedManagedIndexSourceRoot>,
    pub(crate) _work: crate::validation::CustomInvariantWorkMeter,
}

/// The existing participant receives this only from Native's applied draft.
pub struct ManagedIndexSourcePreparation<'a> {
    pub(crate) program: &'a ManagedIndexSourceProgram,
    pub(crate) closure: &'a PreparedManagedIndexScopeClosure,
    pub(crate) views: SourceViews<'a>,
    pub(crate) availability: ManagedIndexSourceAvailability,
    pub(crate) basis: &'a crate::branch::AdmittedRelationalBranchBasis,
    pub(crate) committed_available: bool,
}

impl ManagedIndexSourcePreparation<'_> {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        self.program
    }
    /// This is genuine selected Native custody. It cannot mint Query World
    /// incarnation/READ admission, which the Query resource must bind itself.
    pub fn selected_basis(&self) -> &crate::branch::AdmittedRelationalBranchBasis {
        self.basis
    }
    pub fn scopes(&self) -> impl Iterator<Item = ManagedIndexSourceScope<'_>> {
        self.closure
            .scopes
            .iter()
            .map(|(&scope, changed)| ManagedIndexSourceScope {
                program: self.program,
                scope,
                changed,
                views: &self.views,
                availability: self.availability,
                committed_available: self.committed_available,
            })
    }

    pub(crate) fn check_complete(
        &self,
        roots: &[PreparedManagedIndexSourceRoot],
    ) -> Result<(), Denial> {
        if roots.len() != self.closure.scopes.len() {
            return Err(Denial::IncompletePreparation);
        }
        for ((scope, _), root) in self.closure.scopes.iter().zip(roots) {
            if root.scope != *scope
                || root.program != *self.program
                || root.availability != self.availability
            {
                return Err(Denial::ForeignSource);
            }
            let present = match self.views.after.entity_kind(*scope) {
                Ok(kind) if kind == self.program.scope_kind() => true,
                Err(crate::validation::data::StructuralReadError::RecordUnavailable) => false,
                Ok(_) => return Err(Denial::ForeignSource),
                Err(cause) => {
                    return Err(ReadDenial::Structural(
                        self.views.after.read_budget().refusal(cause),
                    )
                    .into())
                }
            };
            if root.proposed.is_some() != present {
                return Err(Denial::IncompletePreparation);
            }
        }
        self.views
            .after
            .read_budget()
            .check_live()
            .map_err(ReadDenial::Structural)?;
        Ok(())
    }
}
