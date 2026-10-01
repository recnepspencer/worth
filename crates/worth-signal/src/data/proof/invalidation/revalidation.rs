use crate::data::aspect::{Aspect, AspectMask};
use crate::data::output::PartitionSubscription;

use super::binding::{DependencyRevision, PendingDependencyRevalidation, ResolvedDependencyCause};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalDependencyCauseSet {
    basis: CanonicalInvalidationBasis,
    dirty_aspects: AspectMask,
    dirty_scoped_aspects: Vec<(Aspect, PartitionSubscription)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CanonicalInvalidationBasis {
    DependencyCauses(Vec<ResolvedDependencyCause>),
    SourceRecompute(ResolvedDependencyBasis),
    StructuralRecompute(ResolvedDependencyBasis),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CanonicalInvalidationOrigin {
    DependencyCommit,
    SourceRecompute,
    StructuralRecompute,
}

impl CanonicalDependencyCauseSet {
    pub(crate) fn from_dependency_causes(causes: Vec<ResolvedDependencyCause>) -> Self {
        let mut dirty_aspects = AspectMask::EMPTY;
        let mut dirty_scoped_aspects = Vec::new();
        for cause in &causes {
            dirty_aspects.insert(cause.key.aspect);
            dirty_scoped_aspects.extend(
                cause
                    .changed_scopes
                    .as_slice()
                    .iter()
                    .cloned()
                    .map(|scope| (cause.key.aspect, scope)),
            );
        }
        dirty_scoped_aspects.sort_unstable();
        dirty_scoped_aspects.dedup();
        Self {
            basis: CanonicalInvalidationBasis::DependencyCauses(causes),
            dirty_aspects,
            dirty_scoped_aspects,
        }
    }

    pub(crate) fn from_source_recompute(
        revision: DependencyRevision,
        origin_generation: u64,
        dirty_aspects: AspectMask,
        mut dirty_scoped_aspects: Vec<(Aspect, PartitionSubscription)>,
    ) -> Self {
        dirty_scoped_aspects.sort_unstable();
        dirty_scoped_aspects.dedup();
        Self {
            basis: CanonicalInvalidationBasis::SourceRecompute(ResolvedDependencyBasis::new(
                revision,
                origin_generation,
            )),
            dirty_aspects,
            dirty_scoped_aspects,
        }
    }

    pub(crate) fn structural(revision: DependencyRevision) -> Self {
        Self {
            basis: CanonicalInvalidationBasis::StructuralRecompute(ResolvedDependencyBasis::new(
                revision, revision.0,
            )),
            dirty_aspects: AspectMask::EMPTY,
            dirty_scoped_aspects: Vec::new(),
        }
    }

    pub(crate) const fn dirty_aspects(&self) -> AspectMask {
        self.dirty_aspects
    }

    pub(crate) fn dirty_scoped_aspects(&self) -> &[(Aspect, PartitionSubscription)] {
        &self.dirty_scoped_aspects
    }

    /// Bounds the copies and canonical origin sorting performed by lowering.
    pub(crate) fn lowering_copy_work(&self) -> u64 {
        let scope_work = |scope: &PartitionSubscription| {
            (scope.path().total_segment_bytes() as u64)
                .saturating_add(scope.path().depth() as u64)
                .saturating_add(1)
        };
        let mut units = 1_u64;
        for (_, scope) in &self.dirty_scoped_aspects {
            units = units.saturating_add(scope_work(scope));
        }
        if let Some(causes) = self.dependency_causes() {
            units = units.saturating_add(
                (causes.len() as u64)
                    .saturating_mul(causes.len().checked_ilog2().unwrap_or(0) as u64 + 32),
            );
            for cause in causes {
                for scope in [&cause.key.edge_scope, &cause.binding_axes.edge_scope]
                    .into_iter()
                    .flatten()
                {
                    units = units.saturating_add(scope_work(scope));
                }
                for scope in cause.changed_scopes.as_slice() {
                    units = units.saturating_add(scope_work(scope));
                }
            }
        }
        units
    }

    pub(crate) fn lowering_heap_bound(&self) -> Result<u64, crate::data::error::SignalError> {
        use crate::data::retained_storage::{
            RetainedStorageCharge as Charge, RetainedStorageMeasurement, RetainedStoragePreparation,
        };
        let mut measurement = RetainedStoragePreparation::new(usize::MAX);
        let bound = (|| {
            let mut bytes = self
                .dirty_scoped_aspects
                .retained_heap_charge(&mut measurement)?;
            if let Some(causes) = self.dependency_causes() {
                bytes = bytes
                    .checked_add(Charge::capacity::<ResolvedDependencyCause>(causes.len())?)?;
                for cause in causes {
                    bytes = bytes.checked_add(cause.retained_heap_charge(&mut measurement)?)?;
                }
                // Origin evidence is copied across resolve/lower/ready bindings.
                bytes = bytes.checked_add(
                    Charge::capacity::<super::binding::OutputCommitOrdinal>(causes.len())?
                        .checked_mul(4)?,
                )?;
            }
            bytes.checked_mul(3)
        })()
        .map_err(|_| {
            crate::data::error::SignalError::invalid_input(
                "invalidation preparation memory overflow",
            )
        })?;
        Ok(bound.bytes())
    }

    pub(crate) fn is_bound_to_revision(&self, revision: DependencyRevision) -> bool {
        match &self.basis {
            CanonicalInvalidationBasis::DependencyCauses(causes) => causes
                .iter()
                .all(|cause| cause.key.dependency_revision == revision),
            CanonicalInvalidationBasis::SourceRecompute(basis)
            | CanonicalInvalidationBasis::StructuralRecompute(basis) => {
                basis.is_bound_to_revision(revision)
            }
        }
    }

    pub(crate) const fn is_source_recompute(&self) -> bool {
        matches!(self.basis, CanonicalInvalidationBasis::SourceRecompute(_))
    }

    pub(crate) const fn origin(&self) -> CanonicalInvalidationOrigin {
        match self.basis {
            CanonicalInvalidationBasis::DependencyCauses(_) => {
                CanonicalInvalidationOrigin::DependencyCommit
            }
            CanonicalInvalidationBasis::SourceRecompute(_) => {
                CanonicalInvalidationOrigin::SourceRecompute
            }
            CanonicalInvalidationBasis::StructuralRecompute(_) => {
                CanonicalInvalidationOrigin::StructuralRecompute
            }
        }
    }

    pub(crate) fn dependency_causes(&self) -> Option<&[ResolvedDependencyCause]> {
        match &self.basis {
            CanonicalInvalidationBasis::DependencyCauses(causes) => Some(causes),
            CanonicalInvalidationBasis::SourceRecompute(_)
            | CanonicalInvalidationBasis::StructuralRecompute(_) => None,
        }
    }

    pub(crate) const fn origin_generation(&self) -> Option<u64> {
        match &self.basis {
            CanonicalInvalidationBasis::SourceRecompute(basis)
            | CanonicalInvalidationBasis::StructuralRecompute(basis) => {
                Some(basis.origin_generation())
            }
            CanonicalInvalidationBasis::DependencyCauses(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedDependencyBasis {
    dependency_revision: DependencyRevision,
    origin_generation: u64,
}

impl ResolvedDependencyBasis {
    pub(crate) const fn new(
        dependency_revision: DependencyRevision,
        origin_generation: u64,
    ) -> Self {
        Self {
            dependency_revision,
            origin_generation,
        }
    }

    pub(crate) const fn is_bound_to_revision(self, revision: DependencyRevision) -> bool {
        self.dependency_revision.0 == revision.0
    }

    pub(crate) const fn origin_generation(self) -> u64 {
        self.origin_generation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NodeInvalidationInput {
    Pending(PendingDependencyRevalidation),
    Resolved(CanonicalDependencyCauseSet),
    ResolvedNoChange(ResolvedDependencyBasis),
}

impl NodeInvalidationInput {
    pub(crate) fn resolved_dirty_aspects(&self) -> Option<AspectMask> {
        match self {
            Self::Pending(_) => None,
            Self::Resolved(causes) => Some(causes.dirty_aspects()),
            Self::ResolvedNoChange(_) => Some(AspectMask::EMPTY),
        }
    }
}
