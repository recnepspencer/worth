use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisReleaseReceipt,
};

use super::super::read_terminal::WorthQueryGraphReadCompletionParts;
use super::{
    WorthQueryGraphWorkBasis, WorthQueryManagedGraphReadDenial, WorthQueryManagedGraphWorkSession,
};
use crate::domain_computation::provider_session::{
    WorthQueryGraphReadCompletion, WorthQueryGraphReadDependencyEvidence,
    WorthQueryObservedGraphReadWork, WorthQuerySessionGraphReadProof,
};

impl WorthQueryManagedGraphWorkSession {
    pub(in crate::domain_computation) fn graph_read_review(
        &self,
    ) -> &worth_query_admission::facade::graph_read_access::WorthQueryGraphReadPlanReview {
        self.plan
            .graph_read_review()
            .expect("an application-query session owns one reviewed graph-read plan")
    }

    pub(in crate::domain_computation) fn execute_query_read<T>(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
        read: impl FnOnce(
            &mut worth_relational::facade::runtime::RelationalRuntime,
            &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        ) -> T,
    ) -> Result<(T, WorthQuerySessionGraphReadProof), WorthQueryManagedGraphReadDenial> {
        let WorthQueryGraphWorkBasis::Query { identity, port, .. } = &self.basis else {
            return Err(WorthQueryManagedGraphReadDenial::MutationSession);
        };
        if identity != basis || !self.branch.admits_query_basis(basis) {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let output = port
            .execute(&self.binding, read)
            .map_err(|_| WorthQueryManagedGraphReadDenial::ForeignGraph)?;
        Ok((
            output,
            WorthQuerySessionGraphReadProof::new(
                self.identity,
                self.plan.identity(),
                basis.clone(),
            ),
        ))
    }

    pub(in crate::domain_computation) fn complete_query_read(
        self,
        proof: WorthQuerySessionGraphReadProof,
        observed: WorthQueryObservedGraphReadWork,
        basis_release: WorthQueryApplicationBasisReleaseReceipt,
    ) -> Result<WorthQueryGraphReadCompletion, WorthQueryManagedGraphReadDenial> {
        let WorthQueryGraphWorkBasis::Query {
            identity: basis, ..
        } = &self.basis
        else {
            return Err(WorthQueryManagedGraphReadDenial::MutationSession);
        };
        if proof.session != self.identity
            || proof.plan != self.plan.identity()
            || proof.basis != *basis
        {
            return Err(WorthQueryManagedGraphReadDenial::ForeignReadProof);
        }
        if !basis_release.released() || basis_release.identity() != basis {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let dependencies =
            WorthQueryGraphReadDependencyEvidence::bind(self.graph_read_review(), observed);
        let session = self.identity;
        let managed_run = self.managed_run;
        let binding = self.binding;
        let obligation = self.obligation;
        let basis = basis.clone();
        let (plan, review, release) = self
            .plan
            .complete_application_query()
            .ok_or(WorthQueryManagedGraphReadDenial::MutationSession)?;
        if release.scope()
            != worth_query_admission::integration::WorthQueryExecutionCapacityReservationScope::GraphWork
            || release.released_reservation_count() != 1
        {
            return Err(WorthQueryManagedGraphReadDenial::TerminalReleaseMismatch);
        }
        Ok(WorthQueryGraphReadCompletion::new(
            WorthQueryGraphReadCompletionParts {
                session,
                managed_run,
                plan,
                binding,
                obligation,
                basis,
                basis_release,
                review,
                dependencies,
                release,
            },
        ))
    }
}
