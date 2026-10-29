//! Reborrow staged roots through the actual rule observation, never proposal numbers.
use std::sync::Arc;

use crate::derived_relation::{DerivedRelationDenial as Definition, DerivedRelationReadDenial};
use crate::identity::data::EntityId;
use crate::validation::data::{StructuralAspectStateView, StructuralRelationView};
use crate::validation::engine::InvariantSourceAffinity;

use super::preparation::{RootPayload, SourceViews};
use super::{
    ManagedIndexSourceAvailability, ManagedIndexSourcePreparationDenial as Denial,
    ManagedIndexSourceProgram, ManagedIndexSourceProjection, ManagedIndexSourceRows,
};

/// Retained data/support for one exact invocation. No request or loan is retained.
pub struct PreparedManagedIndexSourceRead {
    program: ManagedIndexSourceProgram,
    scope: EntityId,
    before: InvariantSourceAffinity,
    after: InvariantSourceAffinity,
    committed: Option<RootPayload>,
    proposed: Option<RootPayload>,
    availability: ManagedIndexSourceAvailability,
    committed_available: bool,
    audience: crate::validation::data::CustomInvariantSemanticIdentity,
}

/// The payload and Source views are borrowed together under the current audience.
pub struct ManagedIndexSourceRead<'a> {
    prepared: &'a PreparedManagedIndexSourceRead,
    views: SourceViews<'a>,
}

impl PreparedManagedIndexSourceRead {
    pub const fn scope(&self) -> EntityId {
        self.scope
    }
    pub(crate) fn matches_invocation(
        &self,
        scope: EntityId,
        sources: &[ManagedIndexSourceProgram],
        audience: &crate::validation::data::CustomInvariantSemanticIdentity,
    ) -> bool {
        self.scope == scope && self.audience == *audience && sources.contains(&self.program)
    }
}

impl ManagedIndexSourceRead<'_> {
    pub fn committed_root(&self) -> Result<&(dyn std::any::Any + Send + Sync), Denial> {
        if !self.prepared.committed_available {
            return Err(Denial::RootUnavailable);
        }
        self.prepared
            .committed
            .as_deref()
            .ok_or(Denial::RootUnavailable)
    }

    pub fn proposed_root(&self) -> Result<&(dyn std::any::Any + Send + Sync), Denial> {
        self.prepared
            .proposed
            .as_deref()
            .ok_or(Denial::RootUnavailable)
    }

    /// Describes physical publication; a logical suspended root is not public Ready.
    pub const fn physical_availability(&self) -> ManagedIndexSourceAvailability {
        self.prepared.availability
    }

    pub fn committed(&self) -> Result<ManagedIndexSourceRows<'_>, Denial> {
        if !self.prepared.committed_available {
            return Err(Denial::RootUnavailable);
        }
        Ok(self.rows(true))
    }

    pub fn proposed(&self) -> ManagedIndexSourceRows<'_> {
        self.rows(false)
    }

    fn rows(&self, before: bool) -> ManagedIndexSourceRows<'_> {
        ManagedIndexSourceRows {
            scope: self.prepared.scope,
            projection: ManagedIndexSourceProjection {
                program: &self.prepared.program,
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
                audience: self.prepared.program.0.issuer.admit(),
            },
        }
    }
}

pub(crate) fn prepare(
    program: &ManagedIndexSourceProgram,
    scope: EntityId,
    audience: &crate::validation::data::CustomInvariantSemanticIdentity,
    committed: &StructuralRelationView<'_>,
    proposed: &StructuralRelationView<'_>,
) -> Result<PreparedManagedIndexSourceRead, Denial> {
    let (owner, before, after) = association(program, committed, proposed)?;
    let prepared = owner.prepared.get().ok_or(Denial::IncompletePreparation)?;
    let source = prepared
        .programs
        .iter()
        .find(|source| source.program == *program)
        .ok_or(Denial::ForeignSource)?;
    let root = source
        .roots
        .iter()
        .find(|root| root.scope == scope)
        .ok_or(Denial::ForeignSource)?;
    proposed
        .read_budget()
        .claim_retained(size_of::<PreparedManagedIndexSourceRead>() as u64)
        .map_err(DerivedRelationReadDenial::Structural)?;
    Ok(PreparedManagedIndexSourceRead {
        program: program.clone(),
        scope,
        before,
        after,
        committed: root.committed.clone(),
        proposed: root.proposed.clone(),
        availability: root.availability,
        committed_available: owner.physical_basis().materialization_is_complete(),
        audience: audience.clone(),
    })
}

/// Complete union comes from the staged Native program set, never public roots
/// or an application's initiating scope. Retired scopes are cleanup-only.
pub(crate) fn scopes(
    programs: &[ManagedIndexSourceProgram],
    committed: &StructuralRelationView<'_>,
    proposed: &StructuralRelationView<'_>,
) -> Result<Vec<EntityId>, Denial> {
    let mut scopes = std::collections::BTreeSet::new();
    for program in programs {
        let (owner, _, _) = association(program, committed, proposed)?;
        let prepared = owner.prepared.get().ok_or(Denial::IncompletePreparation)?;
        let Some(source) = prepared
            .programs
            .iter()
            .find(|source| source.program == *program)
        else {
            continue;
        };
        for scope in &source.affected_scopes {
            proposed
                .read_budget()
                .checkpoint(1)
                .map_err(DerivedRelationReadDenial::Structural)?;
            match proposed.entity_kind(*scope) {
                Ok(kind) if kind == program.scope_kind() => {
                    proposed
                        .read_budget()
                        .claim_retained(size_of::<EntityId>() as u64)
                        .map_err(DerivedRelationReadDenial::Structural)?;
                    scopes.insert(*scope);
                }
                Err(crate::validation::data::StructuralReadError::RecordUnavailable) => {}
                Ok(_) => return Err(Denial::ForeignSource),
                Err(cause) => {
                    return Err(DerivedRelationReadDenial::Structural(
                        proposed.read_budget().refusal(cause),
                    )
                    .into())
                }
            }
        }
    }
    Ok(scopes.into_iter().collect())
}

pub(crate) fn read<'a>(
    prepared: &'a PreparedManagedIndexSourceRead,
    views: SourceViews<'a>,
) -> Result<ManagedIndexSourceRead<'a>, Denial> {
    let (_, before, after) = association(&prepared.program, &views.before, &views.after)?;
    if before != prepared.before || after != prepared.after {
        return Err(DerivedRelationReadDenial::from(Definition::ForeignObservation).into());
    }
    Ok(ManagedIndexSourceRead { prepared, views })
}

/// No public constructor supplies this association. The views were minted by
/// Native from the actual observation and its strong applied-draft owner.
fn association(
    program: &ManagedIndexSourceProgram,
    committed: &StructuralRelationView<'_>,
    proposed: &StructuralRelationView<'_>,
) -> Result<
    (
        Arc<super::NativeManagedSourceInterpretation>,
        InvariantSourceAffinity,
        InvariantSourceAffinity,
    ),
    Denial,
> {
    for view in [committed, proposed] {
        view.require_derived_relation(&program.0.path)?;
        view.require_derived_change_access(&program.0.path)
            .map_err(|cause| {
                DerivedRelationReadDenial::Structural(view.read_budget().refusal(cause))
            })?;
        view.read_budget()
            .check_live()
            .map_err(DerivedRelationReadDenial::Structural)?;
    }
    let before = &committed
        .structural_source
        .as_ref()
        .ok_or(Denial::ForeignSource)?
        .1;
    let after = &proposed
        .structural_source
        .as_ref()
        .ok_or(Denial::ForeignSource)?
        .1;
    let proposal = after.proposal.as_ref().ok_or(Denial::ForeignSource)?;
    let owner = proposal
        .source_interpretation()
        .ok_or(Denial::ForeignSource)?;
    owner.require_current_registration()?;
    if !before.committed
        || after.committed
        || before.observation != after.observation
        || before.proposal.as_ref() != Some(proposal)
        || before.version != owner.physical_basis().observation().version_id()
        || after.version
            != owner
                .suspension_source_version()
                .unwrap_or(owner.proposed_version())
        || proposal.proposed_version_id() != owner.proposed_version()
    {
        return Err(Denial::ForeignSource);
    }
    Ok((owner, before.clone(), after.clone()))
}
