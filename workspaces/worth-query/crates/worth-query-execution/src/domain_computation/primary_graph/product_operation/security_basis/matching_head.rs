//! Reuse one already issued Query basis only while its World head is current.

use worth_runtime_world::facade::ProductBranchCurrentnessFailure;

use super::*;

pub(in crate::domain_computation::primary_graph) struct SelectedPermissionSecurityStop;

/// A current-head seal whose exact selected field indexes were prepared for
/// the same Native basis. This is the graph-read authority, separate from the
/// earlier permission-only seal.
pub(in crate::domain_computation::primary_graph) struct SelectedPreparedReadSecurityBasis<'basis> {
    basis: WorthQueryProductSecurityBasis<'basis>,
}

impl SelectedPreparedReadSecurityBasis<'_> {
    pub(in crate::domain_computation::primary_graph) fn snapshot_handle(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.basis.snapshot_handle()
    }
}

pub(in crate::domain_computation::primary_graph) trait MatchingHeadScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn admit_matching_head_query_prepared_read_security_basis_with_native_access<'basis>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::basis::WorthQueryProductObservationLease,
        query_basis: &'basis WorthQueryApplicationQueryBasisCustody,
        prepared: &PreparedSelectedReadIndexes,

        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<Option<SelectedPreparedReadSecurityBasis<'basis>>, SelectedPermissionSecurityStop>
    {
        if !prepared.matches_basis(product.relational_basis()) {
            return Ok(None);
        }
        self.admit_matching_head_query_permission_basis_with_native_access(
            execution,
            product,
            query_basis,
            native,
        )
        .map(|sealed| {
            sealed.map(|permission| SelectedPreparedReadSecurityBasis {
                basis: permission.basis,
            })
        })
    }

    fn admit_matching_head_query_permission_basis<'basis>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::basis::WorthQueryProductObservationLease,
        query_basis: &'basis WorthQueryApplicationQueryBasisCustody,
    ) -> Result<
        Option<WorthQuerySelectedPermissionSecurityBasis<'basis>>,
        SelectedPermissionSecurityStop,
    > {
        self.admit_matching_head_query_permission_basis_with_native_access(execution, product, query_basis, &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire())
    }

    fn admit_matching_head_query_permission_basis_with_native_access<'basis>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::basis::WorthQueryProductObservationLease,
        query_basis: &'basis WorthQueryApplicationQueryBasisCustody,

        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        Option<WorthQuerySelectedPermissionSecurityBasis<'basis>>,
        SelectedPermissionSecurityStop,
    > {
        let expected = product.observation();

        // The optional security guard, Query basis, and World head each
        // compare the initialized Product identity. Fund every reached
        // comparison, plus the fixed owner/runtime/live probes, up front.

        let guard_matches = product
            .current_security_guard()
            .is_none_or(|guard| guard == expected);
        if expected.owner_identity() != self.product_runtime().owner.owner_identity()
            || !guard_matches
        {
            return Ok(None);
        }
        let graph = self.primary_graph().ok_or(SelectedPermissionSecurityStop)?;
        let reusable = native.with_runtime_mut(
            graph,
            execution,
            || SelectedPermissionSecurityStop,
            |runtime| Ok(query_basis.can_reuse_security_snapshot_at_native(expected, runtime)),
        )?;
        if !reusable {
            return Ok(None);
        }
        let Some(snapshot) = query_basis.reusable_permission_snapshot() else {
            return Ok(None);
        };
        let current = self
            .product_runtime()
            .owner
            .observation_port()
            .while_product_branch_current(expected, (), |(), _head| true);
        match current {
            Ok(true) => Ok(Some(WorthQuerySelectedPermissionSecurityBasis {
                basis: WorthQueryProductSecurityBasis {
                    _observation: expected.clone(),
                    application_basis: SecurityApplicationBasis::Reused(snapshot),
                    _selected_program: None,
                },
            })),
            Ok(false) | Err(ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(())) => {
                Ok(None)
            }

            Err(ProductBranchCurrentnessFailure::AdmissionDenied { .. }) => {
                Err(SelectedPermissionSecurityStop)
            }
        }
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > MatchingHeadScopedOps<Schema> for HostRuntime
{
}
