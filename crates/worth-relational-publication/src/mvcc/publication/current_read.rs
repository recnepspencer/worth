//! Ordered owner-cell guards for a readonly source and a publication target.
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_branch::internal_port::branch::coordination::RelationalBranchCoordinationGuard;
use worth_relational_branch::internal_port::branch::reference_publication_cell::{
    RelationalBranchPublicationCell, RelationalBranchPublicationStateGuard,
};

pub use worth_relational_contracts::internal_port::mvcc::publication::read_dependency_denial::RelationalPublicationReadDependencyDenial;

/// Coordination precedes state acquisition, preserving the ordinary receiver's
/// stop/latch boundaries. Both kinds of locks follow actual branch identity.
pub struct PublicationReadCoordination<'cell> {
    target: &'cell RelationalBranchPublicationCell,
    source: Option<&'cell AdmittedRelationalBranchBasis>,
    target_guard: RelationalBranchCoordinationGuard<'cell>,
    source_guard: Option<RelationalBranchCoordinationGuard<'cell>>,
}

impl<'cell> PublicationReadCoordination<'cell> {
    pub fn enter(
        target: &'cell RelationalBranchPublicationCell,
        source: Option<&'cell AdmittedRelationalBranchBasis>,
    ) -> Self {
        let separate = source.filter(|source| source.identity() != target.identity());
        let (target_guard, source_guard) = match separate {
            Some(source) if source.identity() < target.identity() => {
                let source_guard = source.owner_inner().publication_cell.coordination().enter();
                (target.coordination().enter(), Some(source_guard))
            }
            Some(source) => {
                let target_guard = target.coordination().enter();
                (
                    target_guard,
                    Some(source.owner_inner().publication_cell.coordination().enter()),
                )
            }
            None => (target.coordination().enter(), None),
        };
        Self {
            target,
            source,
            target_guard,
            source_guard,
        }
    }

    pub(super) fn enter_states(
        &self,
    ) -> Result<
        (
            RelationalBranchPublicationStateGuard<'cell>,
            Option<RelationalBranchPublicationStateGuard<'cell>>,
        ),
        RelationalPublicationReadDependencyDenial,
    > {
        let separate = self
            .source
            .filter(|source| source.identity() != self.target.identity());
        let (target, source) = match separate {
            Some(source) if source.identity() < self.target.identity() => {
                let source = source.owner_inner().publication_cell.enter_state();
                (self.target.enter_state(), Some(source))
            }
            Some(source) => {
                let target = self.target.enter_state();
                (
                    target,
                    Some(source.owner_inner().publication_cell.enter_state()),
                )
            }
            None => (self.target.enter_state(), None),
        };
        if let Some(expected) = self.source {
            let observed = source.as_ref().unwrap_or(&target);
            if !observed.matches_current_basis(expected) {
                return Err(RelationalPublicationReadDependencyDenial::NotCurrent);
            }
        }
        Ok((target, source))
    }

    /// After the source state guard ends, release its coordination while the
    /// ordinary target guard remains live through its original cutover phase.
    pub fn into_target_guard(self) -> RelationalBranchCoordinationGuard<'cell> {
        drop(self.source_guard);
        self.target_guard
    }
}
