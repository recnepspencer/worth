//! Move-only management of the actual branch selector used as an application head.
use super::super::WorthQueryApplicationProductBranchCloseDenial;
use super::{WorthQueryExpectedProductBranch, WorthQueryExpectedProductBranchAdoptionFailure};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCloseReceipt;
use crate::domain_computation::primary_graph::product_operation::ContextOps;
use worth_query_installation::facade::ApplicationSchema;
use {
    crate::domain_computation::primary_graph::WorthQueryApplicationProductBranchCleanupFailure,
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

/// A head issued by consuming a newly created expected-source branch.
///
/// This owns the application's head slot, not a model-instance attachment or a
/// grant to execute domain operations. A copied selector cannot create another owner.
/// ```compile_fail,E0451
/// use worth_query_execution::facade::{
///     primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
///     product::{WorthQueryOwnedProductHead, WorthQueryProductBranch},
/// };
/// use worth_query_installation::facade::ApplicationSchema;
/// fn copied_selector<'r, S: ApplicationSchema>(
///     application: &'r WorthQueryPrimaryGraphApplicationRuntime<S>,
///     branch: WorthQueryProductBranch,
/// ) -> WorthQueryOwnedProductHead<'r, S> {
///     WorthQueryOwnedProductHead { application, branch }
/// }
/// ```
/// The genuine fork supplies the lawful ownership transfer.
/// ```
/// use worth_query_execution::facade::product::{WorthQueryExpectedProductBranch, WorthQueryOwnedProductHead};
/// use worth_query_installation::facade::ApplicationSchema;
/// fn transfer<S: ApplicationSchema>(candidate: WorthQueryExpectedProductBranch<'_, S>)
///     -> WorthQueryOwnedProductHead<'_, S> {
///     candidate.into_owned_head()
/// }
/// ```
#[must_use = "transfer the head to its application owner or close it explicitly"]
pub struct WorthQueryOwnedProductHead<'runtime, Schema> {
    application: &'runtime dyn crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>,
    branch: WorthQueryProductBranch,
}

impl<'runtime, Schema> WorthQueryOwnedProductHead<'runtime, Schema> {
    pub(in crate::domain_computation::primary_graph) fn application(
        &self,
    ) -> &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.application.application_runtime()
    }

    pub(super) fn from_created_branch(
        application: &'runtime dyn crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>,
        branch: WorthQueryProductBranch,
    ) -> Self {
        Self {
            application,
            branch,
        }
    }

    /// Descriptive request target; it cannot reconstruct this move-only owner.
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.branch
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryOwnedProductHead<'runtime, Schema> {
    /// Replace this actual slot through the original Native/World guarded adoption.
    /// A refusal preserves this head and returns the candidate in its original owner.
    /// The previous head remains separately owned and requires its own close.
    pub fn adopt(
        &mut self,
        candidate: WorthQueryExpectedProductBranch<'runtime, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<Self, WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema>> {
        let previous = candidate.adopt(&mut self.branch, phase)?;
        Ok(Self::from_created_branch(self.application, previous))
    }

    pub(in crate::domain_computation::primary_graph) fn adopt_at_deferred(
        &mut self,
        candidate: WorthQueryExpectedProductBranch<'runtime, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        expected: &crate::basis::WorthQueryProductObservationLease,
    ) -> Result<
        (
            Self,
            super::expected_source::ProductBranchAdoptionRelease<'runtime, Schema>,
        ),
        WorthQueryExpectedProductBranchAdoptionFailure<'runtime, Schema>,
    > {
        let (previous, release) = candidate.adopt_at_deferred(&mut self.branch, phase, expected)?;
        Ok((
            Self::from_created_branch(self.application, previous),
            release,
        ))
    }

    /// Close the current head through its existing application lifecycle.
    /// Retirement followed by unfinished cleanup returns that cleanup owner,
    /// rather than an apparently live head that could be closed a second time.
    pub fn close(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryProductBranchCloseReceipt,
        WorthQueryOwnedProductHeadCloseFailure<'runtime, Schema>,
    > {
        match self.application.on_branch(self.branch).close(phase) {
            Ok(receipt) => Ok(receipt),
            Err(WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(failure)) => {
                Err(WorthQueryOwnedProductHeadCloseFailure::Retired {
                    branch: self.branch,
                    failure,
                })
            }
            Err(denial) => {
                Err(WorthQueryOwnedProductHeadCloseFailure::Retained { head: self, denial })
            }
        }
    }
}

/// A refusal before retirement retains the head; a later refusal retains cleanup.
#[must_use = "retain the live head or retry the actual retired cleanup owner"]
pub enum WorthQueryOwnedProductHeadCloseFailure<'runtime, Schema> {
    Retained {
        head: WorthQueryOwnedProductHead<'runtime, Schema>,
        denial: WorthQueryApplicationProductBranchCloseDenial,
    },
    Retired {
        branch: WorthQueryProductBranch,
        failure: WorthQueryApplicationProductBranchCleanupFailure,
    },
}

impl<Schema> std::fmt::Debug for WorthQueryOwnedProductHead<'_, Schema> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryOwnedProductHead")
            .field("branch", &self.branch)
            .finish_non_exhaustive()
    }
}

impl<Schema> std::fmt::Debug for WorthQueryOwnedProductHeadCloseFailure<'_, Schema> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retained { head, denial } => f
                .debug_struct("Retained")
                .field("head", head)
                .field("denial", denial)
                .finish(),
            Self::Retired { branch, failure } => f
                .debug_struct("Retired")
                .field("branch", branch)
                .field("failure", failure)
                .finish(),
        }
    }
}
