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
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(Some(std::mem::size_of::<Self>() + 32))?;
        let Self {
            pending_dependency_revalidation,
            direct_invalidation_basis,
            direct_invalidation_generation: _,
            aspect_version_overrides,
            dirty_partition_scope_payload,
            runtime_artifact_state,
        } = self;
        if let Some(pending) = pending_dependency_revalidation {
            sequence::<crate::data::handle::NodeId>(pending.unresolved_producers().len(), work)?;
        }
        if let Some(basis) = direct_invalidation_basis {
            let scopes = basis.scoped_aspects();
            sequence::<(crate::data::aspect::Aspect, PartitionSubscription)>(scopes.len(), work)?;
            for (_, scope) in scopes {
                scope_copy(scope, work)?;
            }
        }
        aspect_version_overrides.admit_clone_work(work)?;
        sequence::<(crate::data::aspect::Aspect, PartitionSubscription)>(
            dirty_partition_scope_payload.len(),
            work,
        )?;
        for (_, scope) in dirty_partition_scope_payload {
            scope_copy(scope, work)?;
        }
        if let Some(runtime) = runtime_artifact_state {
            runtime_copy(runtime, work)?;
        }
        Ok(())
    }
}

fn runtime_copy(
    runtime: &crate::data::trace::RuntimeArtifactState,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    let crate::data::trace::RuntimeArtifactHot {
        output_hash: _,
        output_change: _,
        recomputed: _,
        dependency_count: _,
        meaningful_input_changes: _,
        changed_partition_count: _,
        propagation_suppressed: _,
        changed_scopes,
    } = runtime.hot();
    scopes_copy(changed_scopes.as_slice(), work)?;
    let crate::data::trace::RuntimeArtifactWarm {
        output_identity,
        continuity_token,
        memoized_origin: _,
        reuse_basis,
        reuse_origin: _,
        reuse_boundary_authority,
        lineage_artifact_id: _,
        merge_authority: _,
    } = runtime.warm();
    string(output_identity.as_ref().map(|v| v.as_str()), work)?;
    string(continuity_token.as_ref().map(|v| v.as_str()), work)?;
    string(
        reuse_basis
            .artifact_family_basis
            .as_ref()
            .map(|v| v.as_str()),
        work,
    )?;
    if let Some(authority) = reuse_boundary_authority {
        let crate::data::reuse::ReuseBoundaryAuthority {
            topology_regime: _,
            tolerance_regime,
            semantic_region_digest: _,
            authority_policy: _,
            artifact_family,
            structural_dependency_basis: _,
            partition_region_basis_digest: _,
            partition_region_basis_count: _,
            strategy_detail: _,
        } = authority;
        comparator(tolerance_regime, work)?;
        string(artifact_family.as_ref().map(|v| v.as_str()), work)?;
    }
    Ok(())
}

fn sequence<T>(count: usize, work: &mut EvaluationWork<'_>) -> Result<(), SignalError> {
    work.reserve(
        count
            .checked_mul(std::mem::size_of::<T>() + 64)
            .and_then(|n| n.checked_add(32))
            .filter(|n| *n <= isize::MAX as usize),
    )
}
fn string(value: Option<&str>, work: &mut EvaluationWork<'_>) -> Result<(), SignalError> {
    work.reserve(value.map_or(Some(1), |value| value.len().checked_add(1)))
}
fn scope_copy(
    scope: &PartitionSubscription,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    string(Some(&scope.partition.0), work)?;
    string(scope.detail.as_deref(), work)
}
fn scopes_copy(
    scopes: &[PartitionSubscription],
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    sequence::<PartitionSubscription>(scopes.len(), work)?;
    for scope in scopes {
        scope_copy(scope, work)?;
    }
    Ok(())
}
fn comparator(
    policy: &VersionComparatorPolicy,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    match policy {
        VersionComparatorPolicy::Custom { key } => string(Some(key), work),
        VersionComparatorPolicy::Exact
        | VersionComparatorPolicy::Tolerance { .. }
        | VersionComparatorPolicy::OutputIdentity
        | VersionComparatorPolicy::Installed { .. } => Ok(()),
    }
}
