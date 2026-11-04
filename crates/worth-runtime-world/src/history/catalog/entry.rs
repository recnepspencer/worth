use std::sync::Arc;

use super::metadata::HistoryMetadataCharge;
use super::CompositeRuntimeWorldCommit;

#[derive(Debug)]
pub(crate) struct CompositeHistoryCatalogEntry {
    pub(super) pins: crate::retention::HistoryRetentionObligation,
    pub(super) commit: Option<Arc<CompositeRuntimeWorldCommit>>,
    pub(super) publication: Option<crate::history::PublicationEvidence>,
    pub(super) metadata_charge: HistoryMetadataCharge,
}

impl CompositeHistoryCatalogEntry {
    pub(crate) fn identity(&self) -> &crate::identity::CompositeCommitIdentity {
        self.commit().identity()
    }

    pub(crate) fn commit(&self) -> &CompositeRuntimeWorldCommit {
        self.commit
            .as_ref()
            .expect("only serving history exposes a commit")
            .as_ref()
    }

    pub(super) fn shared_commit(&self) -> Arc<CompositeRuntimeWorldCommit> {
        Arc::clone(
            self.commit
                .as_ref()
                .expect("only serving history shares a commit"),
        )
    }

    pub(crate) fn retention_keys(&self) -> [crate::inspection::RuntimeWorldRetentionKey; 2] {
        self.pins.retention_keys()
    }

    pub(super) const fn metadata_charge(&self) -> HistoryMetadataCharge {
        self.metadata_charge
    }
}
