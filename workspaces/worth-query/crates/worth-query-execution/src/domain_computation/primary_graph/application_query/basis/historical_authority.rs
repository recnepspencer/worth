use std::marker::PhantomData;
use std::time::Instant;

use super::super::resource_lifecycle::WorthQueryApplicationBasisLease;
use super::super::WorthQueryApplicationBasisIdentity;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::history::RelationalCommitReceipt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationHistoricalRead {
    source: WorthQueryApplicationHistoricalReadSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryApplicationHistoricalReadSource {
    ApplicationCommit {
        provider_runtime_instance_id: u64,
        commit: RelationalCommitReceipt,
        descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
        retention: WorthQueryApplicationHistoricalRetention,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryApplicationHistoricalRetention {
    OwnerLifecycle,
    #[cfg(test)]
    Test(crate::domain_computation::primary_graph::provider::WorthQueryRetainedApplicationCommitBasis),
}

impl WorthQueryApplicationHistoricalRead {
    #[cfg(test)]
    pub(crate) fn current_for_test<Schema>(
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Self {
        let (descriptor, basis) = application
            .product_runtime
            .source
            .observe_branch_basis(&application.relational_branch_identity)
            .expect("test application primary basis remains owner-observable");
        let commit = application.primary_provider.graph.with_runtime(|runtime| {
            runtime
                .history()
                .branch_head_for_observation(&basis.observation())
                .expect("test basis belongs to the application Relational owner")
                .expect("historical test basis requires a committed primary head")
        });
        let retention = application.primary_provider.graph.with_runtime(|runtime| {
            runtime
                .retain_component_basis(&basis)
                .map(crate::domain_computation::primary_graph::provider::WorthQueryRetainedApplicationCommitBasis::for_test)
                .expect("test historical basis remains owner-retainable")
        });
        Self {
            source: WorthQueryApplicationHistoricalReadSource::ApplicationCommit {
                provider_runtime_instance_id: descriptor.runtime_instance_id(),
                commit,
                descriptor,
                retention: WorthQueryApplicationHistoricalRetention::Test(retention),
            },
        }
    }

    pub fn at_application_commit(receipt: &WorthQueryApplicationCommitReceipt) -> Self {
        Self {
            source: WorthQueryApplicationHistoricalReadSource::ApplicationCommit {
                provider_runtime_instance_id: receipt.provider_runtime_instance_id(),
                commit: receipt.commit_reference().clone(),
                descriptor: receipt.basis_descriptor().clone(),
                retention: WorthQueryApplicationHistoricalRetention::OwnerLifecycle,
            },
        }
    }

    pub(super) fn into_source(self) -> WorthQueryApplicationHistoricalReadSource {
        self.source
    }
}

/// Move-only Query authority for one exact historical application snapshot.
pub struct WorthQueryApplicationHistoricalBasis<Schema> {
    pub(super) runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    pub(super) schema_binding: ApplicationSchemaBindingIdentity,
    pub(super) graph_authority_identity: String,
    pub(super) provider_identity: String,
    pub(super) expires_at: Instant,
    pub(super) lease: WorthQueryApplicationBasisLease,
    pub(super) _schema: PhantomData<fn() -> Schema>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationHistoricalBasisReleaseReceipt {
    basis_identity: WorthQueryApplicationBasisIdentity,
    released: bool,
}

impl<Schema> WorthQueryApplicationHistoricalBasis<Schema> {
    pub fn identity(&self) -> &WorthQueryApplicationBasisIdentity {
        self.lease.identity()
    }

    pub fn version_id(&self) -> worth_relational::facade::identity::VersionId {
        self.lease.version_id()
    }

    pub const fn expires_at(&self) -> Instant {
        self.expires_at
    }

    pub fn is_live(&self) -> bool {
        Instant::now() < self.expires_at && self.lease.is_live()
    }

    pub fn release(self) -> WorthQueryApplicationHistoricalBasisReleaseReceipt {
        let release = self.lease.release();
        WorthQueryApplicationHistoricalBasisReleaseReceipt {
            basis_identity: release.identity().clone(),
            released: release.released(),
        }
    }

    pub(super) fn into_lease(self) -> WorthQueryApplicationBasisLease {
        self.lease
    }
}

impl WorthQueryApplicationHistoricalBasisReleaseReceipt {
    pub fn basis_identity(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.basis_identity
    }

    pub const fn released(&self) -> bool {
        self.released
    }
}
