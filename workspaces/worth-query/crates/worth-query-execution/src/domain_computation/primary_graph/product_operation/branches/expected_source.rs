//! An exact selected source remains live with its newly created candidate.
use super::*;
use crate::basis::{
    WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchAdmissionRefusalKind,
    WorthQueryProductBranchCreateError, WorthQueryProductBranchLease,
    WorthQueryProductBranchReadIdentity,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCloseReceipt;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationProductBranchCloseDenial,
    WorthQuerySelectedProductOperation,
};
use worth_query_declaration::facade::branch::WorthQueryProductBranchComponents;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

mod adoption;
mod continuation;
pub(in crate::domain_computation::primary_graph) use adoption::ProductBranchAdoptionRelease;
pub use adoption::{
    WorthQueryExpectedProductBranchAdoptionDenial, WorthQueryExpectedProductBranchAdoptionFailure,
};
pub use continuation::WorthQueryExpectedProductBranchCreationContinuation;

#[must_use = "configure and create the expected-source fork, or drop its admission"]
pub struct WorthQueryExpectedProductBranchFork<'runtime, Schema> {
    application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    fork: WorthQueryProductBranchFork<'runtime>,
}

/// Move-only, runtime-bound expected-source candidate detached from
/// `&'runtime Runtime<Schema>`.
///
/// Constructible only by consuming a genuine `WorthQueryExpectedProductBranch`
/// and rebindable only to the exact runtime allocation that created it.
#[must_use = "rebind the detached expected-source candidate to its original runtime"]
pub(in crate::domain_computation::primary_graph) struct DetachedExpectedProductBranch {
    root_identity: std::sync::Arc<
        worth_query_execution_runtime::facade::execution_runtime::product_world::WorthQueryProductRootIdentity,
    >,
    branch: WorthQueryProductBranch,
    source: WorthQueryProductBranchLease,
}

impl DetachedExpectedProductBranch {
    pub(in crate::domain_computation::primary_graph) const fn branch(
        &self,
    ) -> WorthQueryProductBranch {
        self.branch
    }

    pub(in crate::domain_computation::primary_graph) fn into_detached_head(
        self,
    ) -> super::DetachedOwnedProductHead {
        let Self {
            root_identity,
            branch,
            source,
        } = self;
        drop(source);
        super::DetachedOwnedProductHead::from_detached_branch(root_identity, branch)
    }
}

/// A candidate together with the actual source admission from which it was forked.
/// The branch selector permits ordinary candidate requests; it does not certify
/// adoption as an application model's head.
#[must_use = "retain the expected-source continuation or explicitly close the candidate"]
pub struct WorthQueryExpectedProductBranch<'runtime, Schema> {
    application: WorthQueryRuntimeCustody<'runtime, Schema>,
    branch: WorthQueryProductBranch,
    source: WorthQueryProductBranchLease,
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryExpectedProductBranch<'runtime, Schema> {
    pub(in crate::domain_computation::primary_graph) fn into_detached(
        self,
    ) -> DetachedExpectedProductBranch {
        let root_identity = self
            .application
            .inner
            .runtime
            .product_runtime_ref()
            .root_identity();
        DetachedExpectedProductBranch {
            root_identity,
            branch: self.branch,
            source: self.source,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn from_detached(
        application: WorthQueryRuntimeCustody<'runtime, Schema>,
        detached: DetachedExpectedProductBranch,
    ) -> Self {
        let target = application
            .inner
            .runtime
            .product_runtime_ref()
            .root_identity();
        assert!(
            std::sync::Arc::ptr_eq(&detached.root_identity, &target),
            "detached expected-source candidate may only rebind to its exact original runtime allocation"
        );
        Self {
            application,
            branch: detached.branch,
            source: detached.source,
        }
    }
}

/// A failed close retains both the candidate and its exact source continuation.
pub struct WorthQueryExpectedProductBranchCloseFailure<'runtime, Schema> {
    candidate: WorthQueryExpectedProductBranch<'runtime, Schema>,
    denial: WorthQueryApplicationProductBranchCloseDenial,
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryApplicationProductBranches<'runtime, Schema> {
    /// Consume an admitted selection without selecting a newer source at creation.
    pub fn fork_expected(
        self,
        selected: WorthQuerySelectedProductOperation<'runtime, Schema>,
    ) -> Result<
        WorthQueryExpectedProductBranchFork<'runtime, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        if !std::ptr::eq(self.application, selected.application()) {
            return Err(WorthQueryProductBranchAdmissionDenial::Refused(
                WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner,
            ));
        }
        let (_, source, _) = selected.into_parts();
        Ok(WorthQueryExpectedProductBranchFork {
            application: self.application,
            fork: self
                .branches
                .fork_expected(source)
                .with_application_lifecycle(
                    std::sync::Arc::new(
                        crate::domain_computation::runtime_ports::OutputLineageForkAdapter(
                            self.application
                                .inner
                                .runtime
                                .primary_provider_ref()
                                .graph_ref()
                                .output_lineage_ref()
                                .clone(),
                        ),
                    ),
                    self.application
                        .inner
                        .runtime
                        .primary_provider_ref()
                        .as_ref(),
                    self.application,
                ),
        })
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryExpectedProductBranchFork<'runtime, Schema> {
    pub fn components(
        mut self,
        choose: impl FnOnce(WorthQueryProductBranchComponents) -> WorthQueryProductBranchComponents,
    ) -> Self {
        self.fork = self.fork.components(choose);
        self
    }

    pub fn create(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryExpectedProductBranch<'runtime, Schema>, WorthQueryProductBranchCreateError>
    {
        let custody = WorthQueryRuntimeCustody::Borrowed(self.application);
        self.create_in(custody, phase)
    }

    /// Create the candidate for an owner that reaches this fork's runtime
    /// through its own custody. The fork itself never outlives this call.
    pub(in crate::domain_computation::primary_graph) fn create_in<'owner>(
        self,
        custody: WorthQueryRuntimeCustody<'owner, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryExpectedProductBranch<'owner, Schema>, WorthQueryProductBranchCreateError>
    {
        assert!(custody.same_runtime(self.application));
        let (branch, source) = self.fork.create_retaining_source(phase)?;
        Ok(WorthQueryExpectedProductBranch {
            application: custody,
            branch,
            source,
        })
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryExpectedProductBranch<'runtime, Schema> {
    /// Transfer this created branch into the application's move-only head slot.
    /// This releases the original expected-source lease: the branch is no longer
    /// a pending adoption against that source. Domain model attachment and
    /// current operation authorization remain separate admissions.
    pub fn into_owned_head(self) -> super::WorthQueryOwnedProductHead<'runtime, Schema> {
        let Self {
            application,
            branch,
            source,
        } = self;
        drop(source);
        super::WorthQueryOwnedProductHead::from_created_branch(application, branch)
    }

    pub fn branch(&self) -> WorthQueryProductBranch {
        self.branch
    }

    /// Descriptive comparison data; the live source admission stays in this owner.
    pub fn expected_source(&self) -> WorthQueryProductBranchReadIdentity {
        WorthQueryProductBranchReadIdentity::from_observation(self.source.observation())
    }

    pub(in crate::domain_computation::primary_graph) fn application(
        &self,
    ) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        &self.application
    }

    pub(in crate::domain_computation::primary_graph) fn retained_expected_source(
        &self,
    ) -> crate::basis::WorthQueryProductObservationLease {
        self.source.read_lease()
    }

    pub fn close(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryProductBranchCloseReceipt,
        WorthQueryExpectedProductBranchCloseFailure<'runtime, Schema>,
    > {
        let closed = self.application.on_branch(self.branch).close(phase);
        match closed {
            Ok(receipt) => Ok(receipt),
            Err(denial) => Err(WorthQueryExpectedProductBranchCloseFailure {
                candidate: self,
                denial,
            }),
        }
    }
}

impl<'runtime, Schema> WorthQueryExpectedProductBranchCloseFailure<'runtime, Schema> {
    pub fn denial(&self) -> &WorthQueryApplicationProductBranchCloseDenial {
        &self.denial
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryExpectedProductBranch<'runtime, Schema>,
        WorthQueryApplicationProductBranchCloseDenial,
    ) {
        (self.candidate, self.denial)
    }
}

impl<Schema> std::fmt::Debug for WorthQueryExpectedProductBranchCloseFailure<'_, Schema> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryExpectedProductBranchCloseFailure")
            .field("branch", &self.candidate.branch)
            .field("denial", &self.denial)
            .finish()
    }
}
