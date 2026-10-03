//! Reuse one already issued Query basis only while its World head is current.

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::{
    CurrentProductHead, ProductBranchCurrentnessFailure, RuntimeWorldBranchAdmissionDenial,
    RuntimeWorldServiceDenial,
};

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

pub(in crate::domain_computation::primary_graph) enum SelectedPermissionSecurityStop {
    Admission(CompanionPreflightStop),
    World,
    AccountingOverflow,
}

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

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Consume the second exact selected-index check before sealing this same
    /// issued Product/Query basis against the current World head. A moved head
    /// or different Native basis cannot authorize the graph read.
    pub(in crate::domain_computation::primary_graph) fn admit_matching_head_query_prepared_read_security_basis<
        'basis,
    >(
        &self,
        product: &crate::basis::WorthQueryProductObservationLease,
        query_basis: &'basis WorthQueryApplicationQueryBasisCustody,
        prepared: &PreparedSelectedReadIndexes,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedPreparedReadSecurityBasis<'basis>>, SelectedPermissionSecurityStop>
    {
        admission
            .charge_external_work(2)
            .map_err(SelectedPermissionSecurityStop::Admission)?;
        if !prepared.matches_basis(product.relational_basis()) {
            return Ok(None);
        }
        self.admit_matching_head_query_permission_basis(product, query_basis, admission)
            .map(|sealed| {
                sealed.map(|permission| SelectedPreparedReadSecurityBasis {
                    basis: permission.basis,
                })
            })
    }

    /// Bind the selected Product and already inspected Query snapshot to the
    /// exact installed World head. No new Native snapshot or program authority
    /// is issued. The World callback is a short currentness seal only.
    pub(in crate::domain_computation::primary_graph) fn admit_matching_head_query_permission_basis<
        'basis,
    >(
        &self,
        product: &crate::basis::WorthQueryProductObservationLease,
        query_basis: &'basis WorthQueryApplicationQueryBasisCustody,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        Option<WorthQuerySelectedPermissionSecurityBasis<'basis>>,
        SelectedPermissionSecurityStop,
    > {
        let expected = product.observation();
        let comparison = u64::try_from(CurrentProductHead::comparison_work_bound(expected))
            .map_err(|_| SelectedPermissionSecurityStop::AccountingOverflow)?;
        // The optional security guard, Query basis, and World head each
        // compare the initialized Product identity. Fund every reached
        // comparison, plus the fixed owner/runtime/live probes, up front.
        let comparisons = if product.current_security_guard().is_some() {
            3_u64
        } else {
            2_u64
        };
        let work = comparison
            .checked_mul(comparisons)
            .and_then(|work| work.checked_add(5))
            .ok_or(SelectedPermissionSecurityStop::AccountingOverflow)?;
        admission
            .charge_external_work(work)
            .map_err(SelectedPermissionSecurityStop::Admission)?;
        let guard_matches = product
            .current_security_guard()
            .is_none_or(|guard| guard == expected);
        if expected.owner_identity() != self.product_runtime.owner.owner_identity()
            || !guard_matches
            || !query_basis.can_reuse_security_snapshot_at(expected)
        {
            return Ok(None);
        }
        let Some(snapshot) = query_basis.reusable_permission_snapshot() else {
            return Ok(None);
        };
        let mut admission_stop = None;
        let current = self
            .product_runtime
            .owner
            .observation_port()
            .while_product_branch_current_admitted(
                expected,
                (),
                &mut |work| match admission.charge_external_work(work) {
                    Ok(()) => true,
                    Err(stop) => {
                        admission_stop = Some(stop);
                        false
                    }
                },
                |(), _head| true,
            );
        match current {
            Ok(true) => {
                admission
                    .charge_external_work(2)
                    .map_err(SelectedPermissionSecurityStop::Admission)?;
                Ok(Some(WorthQuerySelectedPermissionSecurityBasis {
                    basis: WorthQueryProductSecurityBasis {
                        _observation: expected.clone(),
                        application_basis: SecurityApplicationBasis::Reused(snapshot),
                        _selected_program: None,
                    },
                }))
            }
            Ok(false) | Err(ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(())) => {
                Ok(None)
            }
            Err(ProductBranchCurrentnessFailure::PreparationDenied(())) => {
                Err(SelectedPermissionSecurityStop::Admission(
                    admission_stop.expect("owner preparation refusal retains its typed stop"),
                ))
            }
            Err(ProductBranchCurrentnessFailure::AdmissionDenied { denial, .. }) => match denial {
                RuntimeWorldServiceDenial::Denied(
                    RuntimeWorldBranchAdmissionDenial::CurrentnessAccountingOverflow,
                ) => Err(SelectedPermissionSecurityStop::AccountingOverflow),
                _ => Err(SelectedPermissionSecurityStop::World),
            },
        }
    }
}
