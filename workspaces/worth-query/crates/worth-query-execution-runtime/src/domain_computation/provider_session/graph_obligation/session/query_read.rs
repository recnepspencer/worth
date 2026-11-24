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

/// World equality's fixed layouts and variable bytes for a session's product
/// and the occurrence a Source context recorded.
fn managed_occurrence_work(
    product: &crate::basis::WorthQueryProductObservationLease,
    occurrence: &crate::basis::WorthQueryProductBranchReadIdentity,
) -> Option<u64> {
    use worth_runtime_world::facade::{
        CompositeBasisKey, CompositeCommitIdentity, ProductBranchIdentity,
        ProductBranchIncarnation, ProductBranchReferenceGeneration,
    };
    [
        size_of::<ProductBranchIdentity>(),
        size_of::<ProductBranchIncarnation>(),
        size_of::<ProductBranchReferenceGeneration>(),
        size_of::<CompositeCommitIdentity>(),
        size_of::<CompositeBasisKey>(),
    ]
    .into_iter()
    .try_fold(0usize, usize::checked_add)
    .and_then(|bytes| bytes.checked_mul(2))
    .and_then(|bytes| bytes.checked_add(product.branch_identity().name().as_str().len()))
    .and_then(|bytes| bytes.checked_add(occurrence.branch_identity().name().as_str().len()))
    .and_then(|bytes| u64::try_from(bytes).ok())
}

/// Issued by the genuine Query session; selectors cannot construct this context.
///
/// A completed Source terminal carries this context into facts that outlive
/// its read, so it names the admitted product occurrence and retains none of
/// it. The sessions that compare it hold the World and component custody.
pub struct WorthQueryQuerySourceContext {
    session: super::WorthQueryGraphWorkSessionIdentity,
    runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    principal: worth_relational::facade::identity::EntityId,
    scope: worth_relational::facade::identity::EntityId,
    occurrence: crate::basis::WorthQueryProductBranchReadIdentity,
}

impl WorthQueryQuerySourceContext {
    pub fn admits_parent<Stop>(
        &self,
        parent: &WorthQueryManagedGraphWorkSession,
        admit: &mut impl FnMut(u64) -> Result<(), Stop>,
        overflow: impl FnOnce() -> Stop,
    ) -> Result<bool, Stop> {
        parent.admits_managed_occurrence(&self.occurrence, admit, overflow)
    }
    pub fn session(&self) -> super::WorthQueryGraphWorkSessionIdentity {
        self.session
    }
    pub fn runtime(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.runtime
    }
    pub fn principal(&self) -> worth_relational::facade::identity::EntityId {
        self.principal
    }
    pub fn scope(&self) -> worth_relational::facade::identity::EntityId {
        self.scope
    }
}

impl WorthQueryManagedGraphWorkSession {
    /// Compare genuine admitted occurrences, independently of each Query's
    /// snapshot issuance. Exact snapshot/session/proof checks remain separate.
    pub fn admits_managed_product<Stop>(
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

    /// As `admits_managed_product`, for the occurrence a Source context
    /// recorded. Its basis key holds the Relational admission that
    /// `issued_from` compares, so the identity drops no axis of that check.
    fn admits_managed_occurrence<Stop>(
        &self,
        occurrence: &crate::basis::WorthQueryProductBranchReadIdentity,
        admit: &mut impl FnMut(u64) -> Result<(), Stop>,
        overflow: impl FnOnce() -> Stop,
    ) -> Result<bool, Stop> {
        let Ok(read) = self.query_read_basis() else {
            return Ok(false);
        };
        admit(managed_occurrence_work(&read._product, occurrence).ok_or_else(overflow)?)?;
        Ok(occurrence.matches_observation(read._product.observation()))
    }

    pub fn retain_query_source_context(
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
            occurrence: crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                _product.observation(),
            ),
        })
    }

    pub fn graph_read_review(
        &self,
    ) -> &worth_query_admission::facade::graph_read_access::WorthQueryGraphReadPlanReview {
        self.plan
            .graph_read_review()
            .expect("an application-query session owns one reviewed graph-read plan")
    }

    pub fn execute_query_read<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &WorthQueryApplicationBasisIdentity,
        read: impl FnOnce(
            &worth_relational::facade::runtime::RelationalRuntime,
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

    pub fn complete_query_read(
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
    pub fn execute_query_read_borrowed<T>(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        read: impl FnOnce(
            &worth_relational::facade::runtime::RelationalRuntime,
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
