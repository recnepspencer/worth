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

mod adoption;
pub use adoption::{
    WorthQueryExpectedProductBranchAdoptionDenial, WorthQueryExpectedProductBranchAdoptionFailure,
};

#[must_use = "configure and create the expected-source fork, or drop its admission"]
pub struct WorthQueryExpectedProductBranchFork<'runtime, Schema> {
    application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    fork: WorthQueryProductBranchFork<'runtime>,
}

/// A candidate together with the actual source admission from which it was forked.
/// The branch selector permits ordinary candidate requests; it does not certify
/// adoption as an application model's head.
#[must_use = "retain the expected-source continuation or explicitly close the candidate"]
pub struct WorthQueryExpectedProductBranch<'runtime, Schema> {
    application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    source: WorthQueryProductBranchLease,
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
                    std::sync::Arc::clone(&self.application.primary_provider.graph.output_lineage),
                    &self.application.primary_provider,
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
        let (branch, source) = self.fork.create_retaining_source(phase)?;
        Ok(WorthQueryExpectedProductBranch {
            application: self.application,
            branch,
            source,
        })
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryExpectedProductBranch<'runtime, Schema> {
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.branch
    }

    /// Descriptive comparison data; the live source admission stays in this owner.
    pub fn expected_source(&self) -> WorthQueryProductBranchReadIdentity {
        WorthQueryProductBranchReadIdentity::from_observation(self.source.observation())
    }

    pub fn close(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryProductBranchCloseReceipt,
        WorthQueryExpectedProductBranchCloseFailure<'runtime, Schema>,
    > {
        match self.application.on_branch(self.branch).close(phase) {
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
