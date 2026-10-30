//! Only the exact Inspection target and current session can produce its terminal.
use super::super::inspection_terminal::{
    WorthQueryInspectionReadCompletion, WorthQueryInspectionReadTerminalDenial,
};
use super::*;
use crate::domain_computation::primary_graph::{
    causal_inspection::WorthQueryApplicationInspectionTarget,
    WorthQueryApplicationBasisReleaseReceipt,
};
use std::sync::Arc;
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity;

pub(in crate::domain_computation) struct WorthQuerySessionInspectionReadProof {
    session: WorthQueryGraphWorkSessionIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
    basis: WorthQueryApplicationBasisIdentity,
    target: Arc<WorthQueryApplicationInspectionTarget>,
}

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn execute_inspection_read<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &Arc<WorthQueryApplicationInspectionTarget>,
        basis: &WorthQueryApplicationBasisIdentity,
        read_graph: impl FnOnce(
            &mut worth_relational::facade::runtime::RelationalRuntime,
            &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        ) -> T,
    ) -> Result<(T, WorthQuerySessionInspectionReadProof), WorthQueryManagedGraphReadDenial> {
        let WorthQueryGraphWorkBasis::ProductRead {
            intent: ProductReadIntent::Inspection(installed),
            read,
        } = &self.basis
        else {
            return Err(WorthQueryManagedGraphReadDenial::ForeignReadIntent);
        };
        if !Arc::ptr_eq(target, installed) {
            return Err(WorthQueryManagedGraphReadDenial::ForeignReadProof);
        }
        if read.identity != *basis || !self.branch.admits_query_basis(basis) {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let output = read
            .port
            .execute(execution, &self.binding, read_graph)
            .map_err(|_| WorthQueryManagedGraphReadDenial::ForeignGraph)?;
        Ok((
            output,
            WorthQuerySessionInspectionReadProof {
                session: self.identity,
                plan: self.plan.identity(),
                basis: basis.clone(),
                target: Arc::clone(target),
            },
        ))
    }

    pub(in crate::domain_computation) fn complete_inspection_read(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        proof: WorthQuerySessionInspectionReadProof,
        basis_release: WorthQueryApplicationBasisReleaseReceipt,
    ) -> Result<WorthQueryInspectionReadCompletion, WorthQueryInspectionReadTerminalDenial> {
        execution
            .consult()
            .map_err(WorthQueryInspectionReadTerminalDenial::Request)?;
        let WorthQueryGraphWorkBasis::ProductRead {
            intent: ProductReadIntent::Inspection(target),
            read,
        } = &self.basis
        else {
            return Err(WorthQueryManagedGraphReadDenial::ForeignReadIntent.into());
        };
        if proof.session != self.identity
            || proof.plan != self.plan.identity()
            || proof.basis != read.identity
            || !Arc::ptr_eq(target, &proof.target)
        {
            return Err(WorthQueryManagedGraphReadDenial::ForeignReadProof.into());
        }
        if !basis_release.custody_released() || basis_release.identity() != &read.identity {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis.into());
        }
        let target = Arc::clone(target);
        let basis = read.identity.clone();
        let (plan, release) = self
            .plan
            .complete_application_inspection()
            .ok_or(WorthQueryManagedGraphReadDenial::ForeignReadIntent)?;
        if release.scope()
            != worth_query_admission::integration::WorthQueryExecutionCapacityReservationScope::GraphWork
            || release.released_reservation_count() != 1
        {
            return Err(WorthQueryManagedGraphReadDenial::TerminalReleaseMismatch.into());
        }
        Ok(WorthQueryInspectionReadCompletion {
            session: self.identity,
            managed_run: self.managed_run,
            plan,
            target,
            basis,
            basis_release,
            release,
        })
    }
}
