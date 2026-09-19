use std::marker::PhantomData;

use super::super::resource_lifecycle::WorthQueryApplicationBasisLease;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_relational::facade::history::RelationalCommitReceipt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryApplicationHistoricalRead {
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
}

impl WorthQueryApplicationHistoricalRead {
    pub(crate) fn at_application_commit(receipt: &WorthQueryApplicationCommitReceipt) -> Self {
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
pub(crate) struct WorthQueryApplicationHistoricalBasis<Schema> {
    pub(super) lease: WorthQueryApplicationBasisLease,
    pub(super) _schema: PhantomData<fn() -> Schema>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryApplicationHistoricalBasisReleaseReceipt {
    released: bool,
}

impl<Schema> WorthQueryApplicationHistoricalBasis<Schema> {
    pub(crate) fn version_id(&self) -> worth_relational::facade::identity::VersionId {
        self.lease.version_id()
    }

    pub(crate) fn release(self) -> WorthQueryApplicationHistoricalBasisReleaseReceipt {
        let release = self.lease.release();
        WorthQueryApplicationHistoricalBasisReleaseReceipt {
            released: release.released(),
        }
    }
}

impl WorthQueryApplicationHistoricalBasisReleaseReceipt {
    pub(crate) const fn released(&self) -> bool {
        self.released
    }
}
