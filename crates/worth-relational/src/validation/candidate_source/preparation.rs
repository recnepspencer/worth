//! Borrow the applied Native Source; seal each root under that same custody.
use super::scope_closure::PreparedManagedIndexScopeClosure;
use super::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
    ManagedIndexSourceProjection,
};
use crate::derived_relation::{
    DerivedRelationDenial as Definition, DerivedRelationReadDenial as ReadDenial,
};
use crate::identity::data::EntityId;
use crate::validation::data::{StructuralAspectStateView, StructuralRelationView};
use std::{any::Any, sync::Arc};

mod projection_rows;
mod row_read;
mod rows;
pub use projection_rows::{
    ManagedIndexSourceProjectionOccurrence, PreparedManagedIndexProjectionRead,
};
pub use row_read::{PreparedManagedIndexSourceOccurrence, PreparedManagedIndexSourceRowRead};
pub use rows::{ManagedIndexSourceOccurrence, ManagedIndexSourceRows};

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
    interpretation: &'a Arc<super::NativeManagedSourceInterpretation>,
    correspondence: Option<&'a super::correspondence::NativeManagedSourceCorrespondence<'a>>,
    pub(crate) program: &'a ManagedIndexSourceProgram,
    pub(crate) scope: EntityId,
    pub(crate) changed: &'a std::collections::BTreeSet<EntityId>,
    pub(crate) views: &'a SourceViews<'a>,
    pub(crate) availability: ManagedIndexSourceAvailability,
    pub(crate) committed_available: bool,
}

impl<'a> ManagedIndexSourceScope<'a> {
    pub fn preparation_metadata(&self) -> Option<super::ManagedIndexSourcePreparationMetadata<'_>> {
        self.interpretation.preparation_metadata()
    }
    /// Native proved complete logical equality at this actual reconstruction.
    /// This returns proposed backing only; the committed physical view stays
    /// unavailable. Query still validates its typed provider installation.
    pub fn corresponding_root(&self) -> Result<Option<RootPayload>, Denial> {
        self.views
            .after
            .read_budget()
            .checkpoint(1)
            .map_err(ReadDenial::Structural)?;
        self.correspondence
            .map_or(Ok(None), |proof| proof.root(self.program, self.scope))
    }
    pub fn read_budget(&self) -> crate::validation::custom_rule::CustomInvariantReadBudget<'_> {
        self.views.after.read_budget()
    }
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
        let changed = self.prepare_changed_occurrences()?;
        self.read_budget()
            .claim_retained(
                changed
                    .len()
                    .checked_mul(size_of::<(
                        Option<ManagedIndexSourceOccurrence>,
                        Option<ManagedIndexSourceOccurrence>,
                    )>())
                    .ok_or(ReadDenial::Definition(Definition::WorkBudgetExceeded))?
                    as u64,
            )
            .map_err(ReadDenial::Structural)?;
        Ok(changed
            .into_iter()
            .map(|(before, after)| (before, after.map(|prepared| prepared.occurrence)))
            .collect())
    }
    /// Changed coordinates retain their actual proposed inverse membership.
    pub fn prepare_changed_occurrences(
        &self,
    ) -> Result<
        Vec<(
            Option<ManagedIndexSourceOccurrence>,
            Option<PreparedManagedIndexSourceOccurrence>,
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
                        Option<PreparedManagedIndexSourceOccurrence>,
                    )>())
                    .ok_or(ReadDenial::Definition(Definition::WorkBudgetExceeded))?
                    as u64,
            )
            .map_err(ReadDenial::Structural)?;
        let before = self.committed()?;
        let after = self.proposed();
        self.changed
            .iter()
            .map(|row| Ok((before.occurrence(*row)?, after.prepare_occurrence(*row)?)))
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
                backing: super::projection::SourceProjectionBacking::Invariant {
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
                    interpretation: self.interpretation,
                    before,
                },
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
            source_changed: !self.changed.is_empty(),
        }
    }
}

pub struct PreparedManagedIndexSourceRoot {
    pub(crate) program: ManagedIndexSourceProgram,
    pub(crate) scope: EntityId,
    pub(crate) committed: Option<RootPayload>,
    pub(crate) proposed: Option<RootPayload>,
    pub(crate) availability: ManagedIndexSourceAvailability,
    pub(crate) source_changed: bool,
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
    pub(crate) interpretation: &'a Arc<super::NativeManagedSourceInterpretation>,
    pub(super) correspondence:
        Option<&'a super::correspondence::NativeManagedSourceCorrespondence<'a>>,
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
    pub fn scopes(&self) -> impl ExactSizeIterator<Item = ManagedIndexSourceScope<'_>> {
        self.closure
            .scopes
            .iter()
            .map(|(&scope, changed)| ManagedIndexSourceScope {
                interpretation: self.interpretation,
                correspondence: self.correspondence,
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
