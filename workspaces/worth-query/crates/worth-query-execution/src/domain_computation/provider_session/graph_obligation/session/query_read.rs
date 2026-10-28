use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisReleaseReceipt,
};

use super::super::read_terminal::WorthQueryGraphReadCompletionParts;
use super::{WorthQueryManagedGraphReadDenial, WorthQueryManagedGraphWorkSession};
use crate::domain_computation::provider_session::{
    WorthQueryGraphReadCompletion, WorthQueryGraphReadDependencyEvidence,
    WorthQueryObservedGraphReadWork, WorthQuerySessionGraphReadProof,
};

fn same_managed_product<Stop>(
    left: &crate::basis::WorthQueryProductObservationLease,
    right: &crate::basis::WorthQueryProductObservationLease,
    admit: &mut impl FnMut(u64) -> Result<(), Stop>,
    overflow: impl FnOnce() -> Stop,
) -> Result<bool, Stop> {
    use worth_relational::facade::branch::{
        RelationalBranchBasisAdmissionIdentity, RelationalBranchBasisDescriptor,
        RelationalBranchIdentity,
    };
    use worth_runtime_world::facade::{
        CompositeBasisKey, CompositeCommitIdentity, ProductBranchIdentity,
        ProductBranchIncarnation, ProductBranchReferenceGeneration, RuntimeWorldOwnerIdentity,
    };
    // Both fixed identity layouts and every variable byte visited by World Eq
    // and Native issued_from are admitted before either actual comparison.
    let fixed = [
        size_of::<RuntimeWorldOwnerIdentity>(),
        size_of::<ProductBranchIdentity>(),
        size_of::<ProductBranchIncarnation>(),
        size_of::<ProductBranchReferenceGeneration>(),
        size_of::<CompositeCommitIdentity>(),
        size_of::<CompositeBasisKey>(),
        size_of::<RelationalBranchBasisDescriptor>(),
        size_of::<RelationalBranchIdentity>(),
        size_of::<RelationalBranchBasisAdmissionIdentity>(),
        size_of::<u64>(),
    ]
    .into_iter()
    .try_fold(0usize, usize::checked_add)
    .and_then(|bytes| bytes.checked_mul(2))
    .and_then(|bytes| u64::try_from(bytes).ok());
    let work = (|| {
        let mut work = fixed?;
        for product in [left, right] {
            work = work.checked_add(
                u64::try_from(product.branch_identity().name().as_str().len()).ok()?,
            )?;
            work = work.checked_add(
                WorthQueryApplicationBasisIdentity::retained_basis_clone_heap_bytes(
                    product.relational_basis(),
                )?,
            )?;
        }
        Some(work)
    })()
    .ok_or_else(overflow)?;
    admit(work)?;
    Ok(left.observation() == right.observation()
        && left
            .relational_basis()
            .observation()
            .issued_from(right.relational_basis()))
}

/// Issued by the genuine Query session; selectors cannot construct this context.
pub(in crate::domain_computation) struct WorthQueryQuerySourceContext {
    session: super::WorthQueryGraphWorkSessionIdentity,
    runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    principal: worth_relational::facade::identity::EntityId,
    scope: worth_relational::facade::identity::EntityId,
    product: crate::basis::WorthQueryProductObservationLease,
}

impl WorthQueryQuerySourceContext {
    pub(in crate::domain_computation) fn admits_parent<Stop>(
        &self,
        parent: &WorthQueryManagedGraphWorkSession,
        admit: &mut impl FnMut(u64) -> Result<(), Stop>,
        overflow: impl FnOnce() -> Stop,
    ) -> Result<bool, Stop> {
        parent.admits_managed_product(&self.product, admit, overflow)
    }
    pub(in crate::domain_computation) fn session(
        &self,
    ) -> super::WorthQueryGraphWorkSessionIdentity {
        self.session
    }
    pub(in crate::domain_computation) fn runtime(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.runtime
    }
    pub(in crate::domain_computation) fn principal(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.principal
    }
    pub(in crate::domain_computation) fn scope(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.scope
    }
}

impl WorthQueryManagedGraphWorkSession {
    /// Compare genuine admitted occurrences, independently of each Query's
    /// snapshot issuance. Exact snapshot/session/proof checks remain separate.
    pub(in crate::domain_computation) fn admits_managed_product<Stop>(
        &self,
        product: &crate::basis::WorthQueryProductObservationLease,
        admit: &mut impl FnMut(u64) -> Result<(), Stop>,
        overflow: impl FnOnce() -> Stop,
    ) -> Result<bool, Stop> {
        let Ok(read) = self.query_read_basis() else {
            return Ok(false);
        };
        same_managed_product(&read._product, product, admit, overflow)
    }

    pub(in crate::domain_computation) fn retain_query_source_context(
        &self,
    ) -> Result<WorthQueryQuerySourceContext, WorthQueryManagedGraphReadDenial> {
        let super::ProductReadBasis {
            identity, _product, ..
        } = self.query_read_basis()?;
        if !self.branch.admits_query_basis(identity) {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let scope = self
            .entity_access_context()
            .ok_or(WorthQueryManagedGraphReadDenial::ForeignReadIntent)?;
        Ok(WorthQueryQuerySourceContext {
            session: self.identity,
            runtime: self.runtime,
            principal: self.principal,
            scope,
            product: _product.retained_clone(),
        })
    }

    pub(in crate::domain_computation) fn graph_read_review(
        &self,
    ) -> &worth_query_admission::facade::graph_read_access::WorthQueryGraphReadPlanReview {
        self.plan
            .graph_read_review()
            .expect("an application-query session owns one reviewed graph-read plan")
    }

    pub(in crate::domain_computation) fn execute_query_read<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &WorthQueryApplicationBasisIdentity,
        read: impl FnOnce(
            &mut worth_relational::facade::runtime::RelationalRuntime,
            &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        ) -> T,
    ) -> Result<(T, WorthQuerySessionGraphReadProof), WorthQueryManagedGraphReadDenial> {
        let super::ProductReadBasis { identity, port, .. } = self.query_read_basis()?;
        if identity != basis || !self.branch.admits_query_basis(basis) {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let output = port
            .execute(execution, &self.binding, read)
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
        let super::ProductReadBasis {
            identity: basis, ..
        } = self.query_read_basis()?;
        if proof.session != self.identity
            || proof.plan != self.plan.identity()
            || proof.basis != *basis
        {
            return Err(WorthQueryManagedGraphReadDenial::ForeignReadProof);
        }
        if !basis_release.custody_released() || basis_release.identity() != basis {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let dependencies =
            WorthQueryGraphReadDependencyEvidence::bind(self.graph_read_review(), observed);
        let session = self.identity;
        let managed_run = self.managed_run;
        let basis = basis.clone();
        let binding = self.binding;
        let obligation = self.obligation;
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

impl WorthQueryManagedGraphWorkSession {
    /// The same query session and exact basis acceptance under an existing
    /// native owner borrow. Completion consumes its ordinary sealed read proof.
    pub(in crate::domain_computation) fn execute_query_read_borrowed<T>(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        read: impl FnOnce(
            &mut worth_relational::facade::runtime::RelationalRuntime,
            &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        ) -> T,
    ) -> Result<(T, WorthQuerySessionGraphReadProof), WorthQueryManagedGraphReadDenial> {
        let super::ProductReadBasis { identity, port, .. } = self.query_read_basis()?;
        if identity != basis || !self.branch.admits_query_basis(basis) {
            return Err(WorthQueryManagedGraphReadDenial::ForeignBasis);
        }
        let output = port
            .execute_borrowed(&self.binding, runtime, read)
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
}
