//! Concrete selected-node payload copy work; not a retained-byte reservation.
mod cold;
use super::NodeWarmData;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::error::SignalError;
use crate::data::output::PartitionSubscription;
use crate::logic::evaluation::EvaluationWork;

impl NodeWarmData {
    pub(crate) fn admit_clone_work(
        &self,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<(), SignalError> {
        work.reserve(Some(std::mem::size_of::<Self>() + 32))?;
        let Self {
            pending_dependency_revalidation,
            direct_invalidation_basis: _,
            direct_invalidation_generation: _,
            aspect_version_overrides: _,
            dirty_partition_scope_payload,
            runtime_artifact_state: _,
        } = self;
        if let Some(pending) = pending_dependency_revalidation {
            sequence::<crate::data::handle::NodeId>(pending.unresolved_producers().len(), work)?;
        }
        // These companions are shared by Arc in a draft; producer mutation
        // admits any copy-on-write detachment separately before evaluation.
        sequence::<(crate::data::aspect::Aspect, PartitionSubscription)>(
            dirty_partition_scope_payload.len(),
            work,
        )?;
        for (_, scope) in dirty_partition_scope_payload {
            scope_copy(scope, work)?;
        }
        Ok(())
    }
}

fn sequence<T>(count: usize, work: &mut EvaluationWork<'_, '_>) -> Result<(), SignalError> {
    work.reserve(
        count
            .checked_mul(std::mem::size_of::<T>() + 64)
            .and_then(|n| n.checked_add(32))
            .filter(|n| *n <= isize::MAX as usize),
    )
}
fn string(value: Option<&str>, work: &mut EvaluationWork<'_, '_>) -> Result<(), SignalError> {
    work.reserve(value.map_or(Some(1), |value| value.len().checked_add(1)))
}
fn scope_copy(
    scope: &PartitionSubscription,
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(), SignalError> {
    work.reserve(
        scope
            .path()
            .depth()
            .checked_mul(std::mem::size_of::<String>()),
    )?;
    for segment in scope.path().segments() {
        string(Some(segment), work)?;
    }
    Ok(())
}

fn scopes_copy(
    scopes: &[PartitionSubscription],
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(), SignalError> {
    sequence::<PartitionSubscription>(scopes.len(), work)?;
    for scope in scopes {
        scope_copy(scope, work)?;
    }
    Ok(())
}

fn comparator(
    policy: &VersionComparatorPolicy,
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(), SignalError> {
    match policy {
        VersionComparatorPolicy::Custom { key } => string(Some(key), work),
        VersionComparatorPolicy::Exact
        | VersionComparatorPolicy::Tolerance { .. }
        | VersionComparatorPolicy::OutputIdentity
        | VersionComparatorPolicy::Installed { .. } => Ok(()),
    }
}
