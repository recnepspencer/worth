//! Exact paired source custody, separate from the descriptive change frontier.
use super::*;
use crate::derived_relation::affected::DerivedAffectedRoots;
use crate::graph_member::NativeMemberKey;
use crate::transactions::data::RecordRef;
use crate::validation::engine::InvariantSourceAffinity;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ChangeIdentity {
    proposed: InvariantSourceAffinity,
    committed: InvariantSourceAffinity,
    pub(super) definition: Handle,
}

/// Complete structural pullback for one actual candidate and installed recipe.
/// These coordinates are neither row-read nor mutation authority. Scalar
/// projection changes additionally require the projection's dependency routing.
pub struct DerivedRelationChangeData {
    pub(super) records: Vec<RecordRef>,
    pub(super) affected: DerivedAffectedRoots,
}
impl DerivedRelationChangeData {
    /// Native's direct record seeds plus its bounded relation frontier. Seeds
    /// can include unchanged endpoints/neighbors; they are not an exact write set.
    pub fn record_seeds(&self) -> &[RecordRef] {
        &self.records
    }
    pub fn affected_roots(&self) -> &BTreeSet<NativeMemberKey> {
        &self.affected.roots
    }
    pub fn affected_targets(&self) -> &BTreeSet<NativeMemberKey> {
        &self.affected.targets
    }
    /// Retains every before/after inverse-prefix acquisition, including empty
    /// adjacency. A consumer must not interpret only positive endpoints as scope.
    pub fn support(&self) -> &[crate::derived_relation::DerivedRelationData] {
        &self.affected.support
    }
}

/// Issued only from the planner's own Native before/after views and touched set.
/// Cloning preserves backing and original payer, never a request or a grant.
#[derive(Clone)]
pub struct PreparedDerivedRelationChanges {
    pub(super) identity: ChangeIdentity,
    pub(super) body: Arc<ChangeBody>,
}
pub(super) struct ChangeBody {
    pub(super) data: DerivedRelationChangeData,
    pub(super) _retention: [crate::validation::CustomInvariantWorkMeter; 2],
}

pub(super) fn identity(
    proposed: &StructuralRelationView<'_>,
    committed: &StructuralRelationView<'_>,
    definition: &Handle,
) -> Result<ChangeIdentity, Denial> {
    proposed.require_derived_relation(definition)?;
    committed.require_derived_relation(definition)?;
    let (_, after) = proposed
        .structural_source
        .as_ref()
        .expect("source admitted");
    let (_, before) = committed
        .structural_source
        .as_ref()
        .expect("source admitted");
    let proposal = after
        .proposal
        .as_ref()
        .ok_or(DefinitionDenial::ForeignObservation)?;
    if after.committed
        || !before.committed
        || after.observation != before.observation
        || after.proposal != before.proposal
        || after.version != proposal.proposed_version_id()
    {
        return Err(DefinitionDenial::ForeignObservation.into());
    }
    Ok(ChangeIdentity {
        proposed: after.clone(),
        committed: before.clone(),
        definition: definition.clone(),
    })
}
