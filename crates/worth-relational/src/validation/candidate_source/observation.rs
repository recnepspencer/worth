//! Reborrow staged roots through the actual rule observation, never proposal numbers.
use std::sync::Arc;
use worth_relational_contracts::internal_port::validation::candidate_source::program::ManagedIndexSourceProgramFieldsPort as _;
use worth_relational_contracts::internal_port::validation::candidate_source::program::ManagedIndexSourceProgramOwnerPort as _;

use crate::derived_relation::definition::DerivedRelationDenial as Definition;
use crate::derived_relation::read_denial::DerivedRelationReadDenial;
use crate::identity::data::EntityId;
use crate::validation::custom_rule::structural_views::StructuralRelationView;
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
    audience: crate::validation::data::rule_id::CustomInvariantSemanticIdentity,
    /// The receiving rule's declared Source set, as the retain that checked
    /// this program's membership saw it; later reads compare it by identity.
    sources: Arc<[ManagedIndexSourceProgram]>,
    /// The selected program's and root's positions in the draft's prepared
    /// Sources: a checked coordinate, never a share of the delta itself.
    coordinate: [usize; 2],
}

/// The payload and Source views are borrowed together under the current audience.
pub struct ManagedIndexSourceRead<'a> {
    prepared: &'a PreparedManagedIndexSourceRead,
    interpretation: Arc<super::NativeManagedSourceInterpretation>,
    views: SourceViews<'a>,
}

impl PreparedManagedIndexSourceRead {
    pub const fn scope(&self) -> EntityId {
        self.scope
    }
    pub(crate) fn matches_invocation(
        &self,
        scope: EntityId,
        sources: &Arc<[ManagedIndexSourceProgram]>,
        audience: &crate::validation::data::rule_id::CustomInvariantSemanticIdentity,
    ) -> bool {
        // Membership was checked once at retain; the same declared set, by
        // identity, carries that proof without a search.
        self.scope == scope && self.audience == *audience && Arc::ptr_eq(&self.sources, sources)
    }
}

impl ManagedIndexSourceRead<'_> {
    pub fn read_budget(&self) -> crate::validation::custom_rule::CustomInvariantReadBudget<'_> {
        self.views.after.read_budget()
    }
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

    /// The provider owner's exact delta for this scope and proposal, on one
    /// side (committed when `before`): rows whose materialized entry left or
    /// changed, or entered or changed, in the invariant's logical
    /// observation. Borrowed from this observation's own draft at the
    /// prepared coordinate, O(1). Crate-private: only the planner's
    /// preparation-time loan reaches it, never evaluation.
    pub(crate) fn changed_members(&self, before: bool) -> Result<&[EntityId], Denial> {
        if before && !self.prepared.committed_available {
            return Err(Denial::RootUnavailable);
        }
        let [program, root] = self.prepared.coordinate;
        let root = self
            .interpretation
            .prepared
            .get()
            .and_then(|staged| staged.programs.get(program))
            .filter(|source| source.program == self.prepared.program)
            .and_then(|source| source.roots.get(root))
            .filter(|root| root.scope == self.prepared.scope)
            .ok_or(Denial::ForeignSource)?;
        Ok(root.changes.side(before))
    }

    fn rows(&self, before: bool) -> ManagedIndexSourceRows<'_> {
        ManagedIndexSourceRows {
            scope: self.prepared.scope,
            projection: ManagedIndexSourceProjection {
                program: &self.prepared.program,
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
                    audience: (*self.prepared.program.owner_field_0()).issuer.admit(),
                    interpretation: &self.interpretation,
                    before,
                },
            },
        }
    }
}

pub(crate) fn prepare(
    program: &ManagedIndexSourceProgram,
    scope: EntityId,
    audience: &crate::validation::data::rule_id::CustomInvariantSemanticIdentity,
    sources: &Arc<[ManagedIndexSourceProgram]>,
    committed: &StructuralRelationView<'_>,
    proposed: &StructuralRelationView<'_>,
) -> Result<PreparedManagedIndexSourceRead, Denial> {
    let (owner, before, after) = association(program, committed, proposed)?;
    let prepared = owner.prepared.get().ok_or(Denial::IncompletePreparation)?;
    let program_at = prepared
        .programs
        .iter()
        .position(|source| source.program == *program)
        .ok_or(Denial::ForeignSource)?;
    let source = &prepared.programs[program_at];
    let root_at = source
        .roots
        .iter()
        .position(|root| root.scope == scope)
        .ok_or(Denial::ForeignSource)?;
    let root = &source.roots[root_at];
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
        committed_available: owner.home_target_step.is_none()
            && owner.physical_basis().materialization_is_complete(),
        audience: audience.clone(),
        sources: Arc::clone(sources),
        coordinate: [program_at, root_at],
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
                Err(crate::validation::data::structural_read::StructuralReadError::RecordUnavailable) => {}
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
    let (interpretation, before, after) =
        association(&prepared.program, &views.before, &views.after)?;
    if before != prepared.before || after != prepared.after {
        return Err(DerivedRelationReadDenial::from(Definition::ForeignObservation).into());
    }
    Ok(ManagedIndexSourceRead {
        prepared,
        views,
        interpretation,
    })
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
    // A target program has no predecessor installation. Both real observation
    // affinities remain checked above; only the proposed target may admit its
    // definition. Committed target reads explicitly remain unavailable.
    for view in [committed, proposed]
        .into_iter()
        .skip(usize::from(owner.home_target_step.is_some()))
    {
        for path in program.paths() {
            view.require_derived_relation(path)?;
            view.require_derived_change_access(path).map_err(|cause| {
                DerivedRelationReadDenial::Structural(view.read_budget().refusal(cause))
            })?;
        }
        view.read_budget()
            .check_live()
            .map_err(DerivedRelationReadDenial::Structural)?;
    }
    Ok((owner, before.clone(), after.clone()))
}
