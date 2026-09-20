use std::num::NonZeroUsize;

use worth_runtime_world::facade::RuntimeWorldOwnerIdentity;

use crate::basis::WorthQueryProductBranch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramAdoptionCoverageDenial {
    EmptyCoverage,
    TargetLimitExceeded { maximum: usize, presented: usize },
    AllocationRejected,
    DuplicateTarget { branch: WorthQueryProductBranch },
    ForeignOrRetiredTarget { branch: WorthQueryProductBranch },
    RegistryUnavailable,
    ForeignApplication,
    OrderedTargetCountMismatch { covered: usize, ordered: usize },
    OrderedTargetMismatch,
}

/// Owner-issued, bounded set of exact live branch occurrences.
///
/// The canonical order is descriptive. Publication order is bound separately
/// so callers must explicitly acknowledge the non-atomic sequence.
pub struct WorthQueryProgramAdoptionCoverage {
    owner: RuntimeWorldOwnerIdentity,
    branches: Box<[WorthQueryProductBranch]>,
}

impl WorthQueryProgramAdoptionCoverage {
    pub(in crate::domain_computation::primary_graph::product_operation) fn issue(
        branches: &[WorthQueryProductBranch],
        maximum: NonZeroUsize,
    ) -> Result<Self, WorthQueryProgramAdoptionCoverageDenial> {
        if branches.is_empty() {
            return Err(WorthQueryProgramAdoptionCoverageDenial::EmptyCoverage);
        }
        if branches.len() > maximum.get() {
            return Err(
                WorthQueryProgramAdoptionCoverageDenial::TargetLimitExceeded {
                    maximum: maximum.get(),
                    presented: branches.len(),
                },
            );
        }
        let mut canonical = Vec::new();
        canonical
            .try_reserve_exact(branches.len())
            .map_err(|_| WorthQueryProgramAdoptionCoverageDenial::AllocationRejected)?;
        canonical.extend_from_slice(branches);
        canonical.sort_unstable();
        if let Some(duplicate) = canonical
            .windows(2)
            .find(|pair| pair[0] == pair[1])
            .map(|pair| pair[0])
        {
            return Err(WorthQueryProgramAdoptionCoverageDenial::DuplicateTarget {
                branch: duplicate,
            });
        }
        Ok(Self {
            owner: canonical[0].occurrence().owner_identity(),
            branches: canonical.into_boxed_slice(),
        })
    }

    pub fn branches(&self) -> &[WorthQueryProductBranch] {
        &self.branches
    }

    pub(in crate::domain_computation::primary_graph::product_operation) fn order(
        self,
        application_owner: RuntimeWorldOwnerIdentity,
        ordered: &[WorthQueryProductBranch],
    ) -> Result<WorthQueryOrderedProgramAdoptionCoverage, WorthQueryProgramAdoptionCoverageDenial>
    {
        if self.owner != application_owner {
            return Err(WorthQueryProgramAdoptionCoverageDenial::ForeignApplication);
        }
        if ordered.len() != self.branches.len() {
            return Err(
                WorthQueryProgramAdoptionCoverageDenial::OrderedTargetCountMismatch {
                    covered: self.branches.len(),
                    ordered: ordered.len(),
                },
            );
        }
        let mut ordered_targets = Vec::new();
        ordered_targets
            .try_reserve_exact(ordered.len())
            .map_err(|_| WorthQueryProgramAdoptionCoverageDenial::AllocationRejected)?;
        ordered_targets.extend_from_slice(ordered);
        let mut canonical_order = Vec::new();
        canonical_order
            .try_reserve_exact(ordered.len())
            .map_err(|_| WorthQueryProgramAdoptionCoverageDenial::AllocationRejected)?;
        canonical_order.extend_from_slice(ordered);
        canonical_order.sort_unstable();
        if canonical_order.as_slice() != self.branches.as_ref() {
            return Err(WorthQueryProgramAdoptionCoverageDenial::OrderedTargetMismatch);
        }
        Ok(WorthQueryOrderedProgramAdoptionCoverage {
            branches: ordered_targets.into_boxed_slice(),
        })
    }
}

/// Exact covered branches in the caller-selected non-atomic publication order.
pub struct WorthQueryOrderedProgramAdoptionCoverage {
    pub(super) branches: Box<[WorthQueryProductBranch]>,
}

impl WorthQueryOrderedProgramAdoptionCoverage {
    pub fn branches(&self) -> &[WorthQueryProductBranch] {
        &self.branches
    }
}
