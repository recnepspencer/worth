use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::runtime::RelationalRuntime;

use super::WorthQueryPrimaryGraphIntegrationHandle;

#[derive(Debug)]
pub(crate) enum WorthQueryPrimaryIndexCurrencyDenial {
    Basis(super::exact_basis_access::WorthQueryExactBasisSnapshotDenial),
    IndexUnavailable(&'static str),
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    /// Makes every primary index current for the exact admitted basis.
    ///
    /// Currency is judged on the observing branch. A fork that has not
    /// committed selects its parent's commit, and a branch-scoped index still
    /// needs a generation of its own there, built from the same exact root.
    pub(crate) fn ensure_primary_indexes_for_basis(
        &self,
        runtime: &mut RelationalRuntime,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<(), WorthQueryPrimaryIndexCurrencyDenial> {
        let observation = basis.observation();
        let Some(source_commit_id) = observation.commit_id() else {
            return Ok(());
        };
        let stale = self
            .primary_index_ids
            .iter()
            .copied()
            .filter(|index_id| {
                runtime
                    .index_access()
                    .published_generation_for_observation(*index_id, &observation)
                    .is_none()
            })
            .collect::<Vec<_>>();
        if stale.is_empty() {
            return Ok(());
        }
        let expected = stale.len();
        let build = runtime.index_authority().refresh_for_basis(
            DerivedIndexBuildRequest {
                source_commit_id,
                branch_id: observation.identity().branch_id().clone(),
                index_ids: stale,
            },
            basis,
            None,
            super::index_maintenance_budget::cold_index_reconstruction_budget(),
        );
        require_complete_build(build, expected)
    }
}

fn require_complete_build(
    build: Result<
        worth_relational::facade::indexes::DerivedIndexMaintenanceOutcome,
        worth_relational::facade::indexes::DerivedIndexMaintenanceDenial,
    >,
    expected: usize,
) -> Result<(), WorthQueryPrimaryIndexCurrencyDenial> {
    let build = build.map_err(|denial| match denial.kind {
        worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::Basis(basis) => {
            WorthQueryPrimaryIndexCurrencyDenial::Basis(index_basis_denial(basis))
        }
        _ => WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(
            "primary graph index reconstruction exceeded its budget or lacked exact authority",
        ),
    })?;
    if build.generations.len() == expected {
        Ok(())
    } else {
        Err(WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(
            "primary graph indexes could not recover to the authoritative head",
        ))
    }
}

fn index_basis_denial(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> super::exact_basis_access::WorthQueryExactBasisSnapshotDenial {
    use super::exact_basis_access::WorthQueryExactBasisSnapshotDenial as QueryDenial;
    use worth_relational::facade::branch::RelationalBranchBasisDenial as Denial;
    match denial {
        Denial::RetentionCapacityExhausted => QueryDenial::RetentionCapacityExhausted,
        Denial::RetentionIdentityExhausted => QueryDenial::RetentionIdentityExhausted,
        Denial::SnapshotIdentityExhausted => QueryDenial::SnapshotIdentityExhausted,
        Denial::MaterializationUnavailable => QueryDenial::BranchMaterializationSuspended,
        Denial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        } => QueryDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        },
        _ => QueryDenial::BranchObservationUnavailable,
    }
}
