use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisReleaseReceipt,
};
use worth_query_admission::{
    facade::{
        graph_obligation::WorthQueryGraphWorkPlanIdentity,
        graph_read_access::WorthQueryGraphReadPlanReview,
    },
    integration::WorthQueryExecutionCapacityReleaseReceipt,
};
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledGraphObligationSetIdentity,
};

use super::{
    WorthQueryGraphReadDependencyEvidence, WorthQueryGraphWorkManagedRunIdentity,
    WorthQueryGraphWorkSessionIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadCompletion {
    session: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
    binding: ApplicationSchemaBindingIdentity,
    obligation: WorthQueryInstalledGraphObligationSetIdentity,
    basis: WorthQueryApplicationBasisIdentity,
    basis_release: WorthQueryApplicationBasisReleaseReceipt,
    review: WorthQueryGraphReadPlanReview,
    dependencies: WorthQueryGraphReadDependencyEvidence,
    release: WorthQueryExecutionCapacityReleaseReceipt,
}

pub(super) struct WorthQueryGraphReadCompletionParts {
    pub session: WorthQueryGraphWorkSessionIdentity,
    pub managed_run: WorthQueryGraphWorkManagedRunIdentity,
    pub plan: WorthQueryGraphWorkPlanIdentity,
    pub binding: ApplicationSchemaBindingIdentity,
    pub obligation: WorthQueryInstalledGraphObligationSetIdentity,
    pub basis: WorthQueryApplicationBasisIdentity,
    pub basis_release: WorthQueryApplicationBasisReleaseReceipt,
    pub review: WorthQueryGraphReadPlanReview,
    pub dependencies: WorthQueryGraphReadDependencyEvidence,
    pub release: WorthQueryExecutionCapacityReleaseReceipt,
}

impl WorthQueryGraphReadCompletion {
    pub(super) fn new(parts: WorthQueryGraphReadCompletionParts) -> Self {
        Self {
            session: parts.session,
            managed_run: parts.managed_run,
            plan: parts.plan,
            binding: parts.binding,
            obligation: parts.obligation,
            basis: parts.basis,
            basis_release: parts.basis_release,
            review: parts.review,
            dependencies: parts.dependencies,
            release: parts.release,
        }
    }

    pub const fn session_identity(&self) -> WorthQueryGraphWorkSessionIdentity {
        self.session
    }

    pub const fn managed_run_identity(&self) -> WorthQueryGraphWorkManagedRunIdentity {
        self.managed_run
    }

    pub const fn plan_identity(&self) -> WorthQueryGraphWorkPlanIdentity {
        self.plan
    }

    pub const fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }

    pub const fn obligation_identity(&self) -> &WorthQueryInstalledGraphObligationSetIdentity {
        &self.obligation
    }

    pub const fn basis_identity(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.basis
    }

    pub const fn basis_release(&self) -> &WorthQueryApplicationBasisReleaseReceipt {
        &self.basis_release
    }

    pub const fn review(&self) -> &WorthQueryGraphReadPlanReview {
        &self.review
    }

    pub const fn dependencies(&self) -> &WorthQueryGraphReadDependencyEvidence {
        &self.dependencies
    }

    pub const fn release(&self) -> &WorthQueryExecutionCapacityReleaseReceipt {
        &self.release
    }
}

mod source;
pub(in crate::domain_computation) use source::WorthQueryCompletedSourceRead;
