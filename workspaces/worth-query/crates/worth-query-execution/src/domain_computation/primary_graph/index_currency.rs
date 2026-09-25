use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::runtime::RelationalRuntime;

use super::WorthQueryPrimaryGraphIntegrationHandle;

#[derive(Debug)]
pub(crate) enum WorthQueryPrimaryIndexCurrencyDenial {
    Basis(super::exact_basis_access::WorthQueryExactBasisSnapshotDenial),
    IndexUnavailable(&'static str),
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    pub(crate) fn ensure_primary_indexes_for_basis(
        &self,
        runtime: &mut RelationalRuntime,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<(), WorthQueryPrimaryIndexCurrencyDenial> {
        let observation = basis.observation();
        let Some(head) = observation.commit_receipt().cloned() else {
            return Ok(());
        };
        if self.primary_indexes_are_current(runtime, &head) {
            return Ok(());
        }
        let build = runtime.index_authority().refresh_for_basis(
            DerivedIndexBuildRequest {
                source_commit_id: head.commit_id,
                branch_id: head.branch_id,
                index_ids: self.primary_index_ids.to_vec(),
            },
            basis,
            None,
            super::index_maintenance_budget::cold_index_reconstruction_budget(),
        );
        self.require_complete_build(build)
    }

    fn primary_indexes_are_current(
        &self,
        runtime: &RelationalRuntime,
        head: &RelationalCommitReceipt,
    ) -> bool {
        self.primary_index_ids.iter().all(|index_id| {
            runtime
                .index_access()
                .published_generation_for_commit(*index_id, head)
                .is_some()
        })
    }

    fn require_complete_build(
        &self,
        build: Result<
            worth_relational::facade::indexes::DerivedIndexMaintenanceOutcome,
            worth_relational::facade::indexes::DerivedIndexMaintenanceDenial,
        >,
    ) -> Result<(), WorthQueryPrimaryIndexCurrencyDenial> {
        let build = build.map_err(|denial| match denial.kind {
            worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::Basis(basis) => {
                WorthQueryPrimaryIndexCurrencyDenial::Basis(index_basis_denial(basis))
            }
            _ => WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(
                "primary graph index reconstruction exceeded its budget or lacked exact authority",
            ),
        })?;
        if build.generations.len() == self.primary_index_ids.len() {
            Ok(())
        } else {
            Err(WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(
                "primary graph indexes could not recover to the authoritative head",
            ))
        }
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
