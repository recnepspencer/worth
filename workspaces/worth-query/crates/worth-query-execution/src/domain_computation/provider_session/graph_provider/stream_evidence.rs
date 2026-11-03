use std::sync::Arc;

use super::call_identity::WorthQueryGraphCallAuthorityIdentity;
use super::materialization::WorthQueryGraphReadMaterialization;
use super::{
    WorthQueryExecutionGraphReadProduct, WorthQueryGraphProviderCall, WorthQueryGraphReadMaterial,
};

#[derive(Debug, PartialEq)]
pub struct WorthQueryExecutionGraphReadStreamEvidence {
    authority_identity: WorthQueryGraphCallAuthorityIdentity,
    identity: Arc<str>,
    call_identity: Arc<str>,
    provider_session_identity: Arc<str>,
    canonical_query_digest: Arc<str>,
    basis_identity: Arc<str>,
    snapshot_identity: Arc<str>,
    chunk_count: u64,
    row_count: u64,
    retained_bytes: usize,
    product: WorthQueryExecutionGraphReadProduct,
}

impl WorthQueryExecutionGraphReadStreamEvidence {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn call_identity(&self) -> &str {
        &self.call_identity
    }

    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }

    pub fn canonical_query_digest(&self) -> &str {
        &self.canonical_query_digest
    }

    pub fn basis_identity(&self) -> &str {
        &self.basis_identity
    }

    pub fn snapshot_identity(&self) -> &str {
        &self.snapshot_identity
    }

    pub const fn chunk_count(&self) -> u64 {
        self.chunk_count
    }

    pub const fn row_count(&self) -> u64 {
        self.row_count
    }

    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub fn product(&self) -> &WorthQueryExecutionGraphReadProduct {
        &self.product
    }

    pub(super) const fn authority_identity(&self) -> WorthQueryGraphCallAuthorityIdentity {
        self.authority_identity
    }
}

pub(crate) struct WorthQueryGraphReadStreamAccumulator {
    materialization: WorthQueryGraphReadMaterialization,
}

impl WorthQueryGraphReadStreamAccumulator {
    pub(crate) fn new(_call: &WorthQueryGraphProviderCall) -> Self {
        Self {
            materialization: WorthQueryGraphReadMaterialization::default(),
        }
    }

    pub(crate) const fn chunk_node_allocation_bytes() -> usize {
        WorthQueryGraphReadMaterialization::node_allocation_bytes()
    }

    pub(crate) const fn stream_allocation_bytes() -> usize {
        std::mem::size_of::<WorthQueryExecutionGraphReadStreamEvidence>()
            .saturating_add(std::mem::size_of::<usize>().saturating_mul(2))
    }

    pub(crate) fn admit_chunk(&mut self, material: WorthQueryGraphReadMaterial) -> usize {
        self.materialization.push(material)
    }

    pub(crate) const fn retained_bytes(&self) -> usize {
        self.materialization.retained_bytes()
    }

    pub(crate) fn finish(
        self,
        call: &WorthQueryGraphProviderCall,
    ) -> WorthQueryExecutionGraphReadStreamEvidence {
        let chunk_count = self.materialization.chunk_count();
        let row_count = u64::try_from(self.materialization.row_count()).unwrap_or(u64::MAX);
        let retained_bytes = self
            .materialization
            .retained_bytes()
            .saturating_add(Self::stream_allocation_bytes());
        let product = WorthQueryExecutionGraphReadProduct::seal_materialization(
            call,
            self.materialization.restore_emission_order(),
        );
        let call_identity = call.call_identity_arc();
        WorthQueryExecutionGraphReadStreamEvidence {
            authority_identity: call.authority_identity(),
            identity: Arc::clone(&call_identity),
            call_identity,
            provider_session_identity: call.provider_session_identity_arc(),
            canonical_query_digest: call.canonical_query_digest_arc(),
            basis_identity: call.basis_identity_arc(),
            snapshot_identity: call.snapshot_identity_arc(),
            chunk_count,
            row_count,
            retained_bytes,
            product,
        }
    }
}
