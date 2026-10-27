//! Resource graph reads and their terminal consume only their own session proof.
use super::super::resource_terminal::WorthQueryResourceReadCompletion;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationBasisReleaseReceipt;
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity;
use worth_query_installation::facade::WorthQueryInstalledResourceAccessContract;

pub(in crate::domain_computation) struct WorthQuerySessionResourceReadProof {
    session: WorthQueryGraphWorkSessionIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
    basis: WorthQueryApplicationBasisIdentity,
    contract: std::sync::Arc<WorthQueryInstalledResourceAccessContract>,
}

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn execute_resource_read<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &WorthQueryApplicationBasisIdentity,
        read_graph: impl FnOnce(
            &mut worth_relational::facade::runtime::RelationalRuntime,
            &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        ) -> T,
    ) -> Result<(T, WorthQuerySessionResourceReadProof), WorthQueryManagedGraphReadDenial> {
        let WorthQueryGraphWorkBasis::ProductRead {
            intent: ProductReadIntent::Resource(contract),
            read,
        } = &self.basis
        else {
            return Err(WorthQueryManagedGraphReadDenial::ForeignReadIntent);
        };
        if read.identity != *basis || !self.branch.admits_query_basis(basis) {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let output = read
            .port
            .execute(execution, &self.binding, read_graph)
            .map_err(|_| WorthQueryManagedGraphReadDenial::ForeignGraph)?;
        Ok((
            output,
            WorthQuerySessionResourceReadProof {
                session: self.identity,
                plan: self.plan.identity(),
                basis: basis.clone(),
                contract: std::sync::Arc::clone(contract),
            },
        ))
    }

    pub(in crate::domain_computation) fn complete_resource_read(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        proof: WorthQuerySessionResourceReadProof,
        basis_release: WorthQueryApplicationBasisReleaseReceipt,
    ) -> Result<WorthQueryResourceReadCompletion, WorthQueryManagedGraphReadDenial> {
        execution
            .check_live()
            .map_err(WorthQueryManagedGraphReadDenial::RequestStop)?;
        let WorthQueryGraphWorkBasis::ProductRead {
            intent: ProductReadIntent::Resource(contract),
            read,
        } = &self.basis
        else {
            return Err(WorthQueryManagedGraphReadDenial::ForeignReadIntent);
        };
        if proof.session != self.identity
            || proof.plan != self.plan.identity()
            || proof.basis != read.identity
            || !std::sync::Arc::ptr_eq(contract, &proof.contract)
        {
            return Err(WorthQueryManagedGraphReadDenial::ForeignReadProof);
        }
        if !basis_release.custody_released() || basis_release.identity() != &read.identity {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let contract = std::sync::Arc::clone(contract);
        let basis = read.identity.clone();
        let (plan, release) = self
            .plan
            .complete_application_resource()
            .ok_or(WorthQueryManagedGraphReadDenial::ForeignReadIntent)?;
        if release.scope()
            != worth_query_admission::integration::WorthQueryExecutionCapacityReservationScope::GraphWork
            || release.released_reservation_count() != 1
        {
            return Err(WorthQueryManagedGraphReadDenial::TerminalReleaseMismatch);
        }
        Ok(WorthQueryResourceReadCompletion {
            session: self.identity,
            managed_run: self.managed_run,
            plan,
            contract,
            basis,
            basis_release,
            release,
        })
    }
}
